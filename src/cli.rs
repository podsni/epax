use std::path::PathBuf;

use clap::{Parser, Subcommand};

/// epax — a fast, native, cross-platform archive tool.
///
/// Compress and extract zip, 7z, gz, bz2, zst and tar archives, and extract
/// rar archives, from a single self-contained binary. No external tools are
/// needed at runtime.
///
/// The format is detected automatically from the file extension (for example
/// `.zip`, `.7z`, `.tar.gz`, `.tgz`, `.tar.zst`); use `--format` to override it.
///
/// RAR archives can only be EXTRACTED — the format is proprietary and cannot be
/// created by any open-source tool.
#[derive(Parser, Debug)]
#[command(name = "epax", version, about, long_about = None)]
#[command(propagate_version = true)]
#[command(after_long_help = MAIN_EXAMPLES)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

const MAIN_EXAMPLES: &str = "\
EXAMPLES:
  # Create archives (format inferred from the output name)
  epax compress backup.zip ./my-project
  epax compress release.tar.zst ./bin ./assets README.md
  epax c docs.7z ./docs                       # 'c' is an alias for compress

  # Extract archives (use -o to choose where)
  epax extract backup.zip                     # into the current directory
  epax extract release.tar.zst -o ./out
  epax x photos.rar -o ./photos               # 'x' is an alias for extract

  # Inspect without extracting
  epax list release.tar.zst                   # aliases: l, ls

  # Single-file streams behave like the standard gzip/bzip2/zstd tools
  epax compress data.csv.gz data.csv          # -> data.csv.gz (no tar)
  epax extract data.csv.gz                    # -> data.csv

SUPPORTED FORMATS:
  zip  7z  gz  bz2  zst  tar      compress + extract
  rar                             extract only (proprietary format)

TIP: run `epax help <command>` (e.g. `epax help compress`) for full details.";

#[derive(Subcommand, Debug)]
pub enum Command {
    /// Create an archive from files and/or directories.
    ///
    /// The archive format is chosen from the OUTPUT file extension unless
    /// `--format` is given. Directories are added recursively, preserving their
    /// relative paths. For the single-stream formats (gz/bz2/zst), multiple
    /// inputs or a directory are packed into a tar container automatically
    /// (name it `.tar.gz`, `.tar.zst`, …); a single file given a bare `.gz` /
    /// `.bz2` / `.zst` name is compressed directly, like the `gzip` tool.
    #[command(visible_alias = "c")]
    #[command(after_long_help = COMPRESS_EXAMPLES)]
    Compress {
        /// Path of the archive to create (e.g. out.zip, out.tar.zst, out.7z).
        /// Its extension selects the format unless --format is given.
        output: PathBuf,

        /// One or more files and/or directories to add.
        /// Directories are archived recursively.
        #[arg(required = true)]
        inputs: Vec<PathBuf>,

        /// Force a format instead of detecting it from the output extension.
        /// One of: zip, 7z, gz, bz2, zst, tar.
        #[arg(short, long, value_name = "FMT")]
        format: Option<String>,

        /// Compression level. Range depends on the format and out-of-range
        /// values are clamped: gzip/zip 0-9 (default 6), bzip2 1-9 (default 6),
        /// zstd 1-22 (default 3). Ignored by tar.
        #[arg(short, long, value_name = "N")]
        level: Option<i32>,

        /// Print the name of each entry as it is added.
        #[arg(short, long)]
        verbose: bool,
    },

    /// Extract an archive into a directory.
    ///
    /// The format is detected from the ARCHIVE extension unless `--format` is
    /// given. Entry paths are sanitized to prevent writing outside the output
    /// directory (protection against path-traversal / zip-slip).
    #[command(visible_alias = "x")]
    #[command(after_long_help = EXTRACT_EXAMPLES)]
    Extract {
        /// Path of the archive to extract.
        archive: PathBuf,

        /// Directory to extract into; created if it does not exist.
        #[arg(short, long, default_value = ".", value_name = "DIR")]
        output: PathBuf,

        /// Force a format instead of detecting it from the archive extension.
        /// One of: zip, 7z, gz, bz2, zst, tar, rar.
        #[arg(short, long, value_name = "FMT")]
        format: Option<String>,

        /// Print the name of each entry as it is extracted.
        #[arg(short, long)]
        verbose: bool,
    },

    /// List the contents of an archive without extracting it.
    ///
    /// Prints one line per entry as `SIZE  NAME`, followed by a count. Sizes are
    /// uncompressed bytes where the format records them.
    #[command(visible_aliases = ["l", "ls"])]
    List {
        /// Path of the archive to inspect.
        archive: PathBuf,

        /// Force a format instead of detecting it from the archive extension.
        /// One of: zip, 7z, gz, bz2, zst, tar, rar.
        #[arg(short, long, value_name = "FMT")]
        format: Option<String>,
    },
}

const COMPRESS_EXAMPLES: &str = "\
EXAMPLES:
  epax compress backup.zip ./my-project
  epax compress site.tar.gz ./public index.html
  epax compress release.tar.zst ./bin ./assets -l 19   # high zstd level
  epax compress logs.7z ./logs -v                       # verbose
  epax compress data.csv.gz data.csv                    # bare single-file gzip

NOTES:
  * Output extension picks the format: .zip .7z .tar .tar.gz/.tgz
    .tar.bz2/.tbz2 .tar.zst/.tzst, or bare .gz/.bz2/.zst for one file.
  * RAR output is rejected: the format cannot be created (exit code 2).";

const EXTRACT_EXAMPLES: &str = "\
EXAMPLES:
  epax extract backup.zip                  # into current directory
  epax extract release.tar.zst -o ./out    # into ./out
  epax extract archive.7z -o /tmp/unpacked -v
  epax extract photos.rar -o ./photos      # RAR is extract-only

NOTES:
  * Use --format to override detection for oddly-named files,
    e.g.  epax extract blob --format zst -o ./out";
