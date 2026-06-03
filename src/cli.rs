use std::path::PathBuf;

use clap::{Parser, Subcommand};

/// epax — a fast, native, cross-platform archive tool.
///
/// Compress and extract zip, 7z, gz, bz2, zst and tar archives, and extract
/// rar archives, from a single self-contained binary. No external tools are
/// needed at runtime.
///
/// FORMAT DETECTION
///   compress: format inferred from the output file extension (.zip, .7z,
///             .tar.gz, .tgz, .tar.zst, …). If the output has no recognized
///             archive extension, the first argument is treated as an input
///             and an output name is generated automatically (e.g. "aku.md"
///             → "aku.zip").
///   extract:  format inferred from the archive extension; falls back to
///             magic-byte detection for files with unknown/missing extensions.
///
/// RAR archives can only be EXTRACTED — the format is proprietary and cannot
/// be created by any open-source tool.
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
  # Compress — format from output extension
  epax compress backup.zip ./my-project
  epax compress release.tar.zst ./bin ./assets README.md
  epax c docs.7z ./docs                         # 'c' alias

  # Compress — auto output name (no archive extension given)
  epax compress aku.md                          # → aku.zip
  epax c report.pdf                             # → report.zip
  epax c notes.txt --format 7z                  # → notes.7z (forced format)

  # Extract — format from extension
  epax extract backup.zip
  epax extract release.tar.zst -o ./out
  epax x photos.rar -o ./photos                 # 'x' or 'e' alias
  epax e archive.7z -o /tmp/out

  # Extract — magic-byte detection (no recognized extension)
  epax extract myarchive                        # sniffs format from file header
  epax e blob -o ./unpacked

  # Inspect without extracting
  epax list release.tar.zst                     # aliases: l, ls

  # Single-file stream — no tar container
  epax compress data.csv.gz data.csv            # → data.csv.gz
  epax extract data.csv.gz                      # → data.csv

SUPPORTED FORMATS:
  zip  7z  gz  bz2  zst  tar      compress + extract
  rar                             extract only (proprietary format)

MANAGE:
  epax update                   # update to the latest release
  epax uninstall                # uninstall epax from this system

TIP: run `epax help <command>` (e.g. `epax help compress`) for full details.";

#[derive(Subcommand, Debug)]
pub enum Command {
    /// Create an archive from files and/or directories.
    ///
    /// The archive format is chosen from the OUTPUT file extension unless
    /// `--format` is given. Directories are added recursively, preserving
    /// their relative paths.
    ///
    /// AUTO OUTPUT NAME
    ///   If OUTPUT has no recognized archive extension and --format is not
    ///   given, OUTPUT is treated as an additional input and the output name
    ///   is generated automatically: <stem-of-first-input>.zip
    ///
    ///   Examples:
    ///     epax compress aku.md        →  aku.zip  (aku.md as input)
    ///     epax c notes.txt dir/       →  notes.zip
    ///     epax c doc.pdf --format 7z  →  doc.7z
    ///
    /// STREAM FORMATS (gz / bz2 / zst)
    ///   Multiple inputs or a directory → packed into a tar container first
    ///   (.tar.gz, .tar.zst, …). A single file with a bare .gz/.bz2/.zst
    ///   name is compressed directly, like the standard gzip tool.
    #[command(visible_alias = "c")]
    #[command(after_long_help = COMPRESS_EXAMPLES)]
    Compress {
        /// Path of the archive to create, OR a file/directory to compress
        /// (auto-output mode: output name is derived from this path + .zip).
        output: PathBuf,

        /// Additional files and/or directories to add.
        /// Directories are archived recursively.
        #[arg()]
        inputs: Vec<PathBuf>,

        /// Force a specific format instead of detecting from the output
        /// extension. One of: zip, 7z, gz, bz2, zst, tar.
        /// In auto-output mode this also changes the output extension.
        #[arg(short, long, value_name = "FMT")]
        format: Option<String>,

        /// Compression level (clamped to format range):
        /// gzip/zip 0-9 (default 6), bzip2 1-9 (default 6),
        /// zstd 1-22 (default 3). Ignored by tar and 7z (always LZMA2).
        #[arg(short, long, value_name = "N")]
        level: Option<i32>,

        /// Print the name of each entry as it is added.
        #[arg(short, long)]
        verbose: bool,
    },

    /// Extract an archive into a directory.
    ///
    /// FORMAT DETECTION
    ///   1. From the --format flag (if given).
    ///   2. From the archive file extension (.zip, .tar.gz, .rar, …).
    ///   3. From the file's magic bytes — works even when the extension is
    ///      missing or wrong (e.g. a zip renamed to .dat is still extracted
    ///      correctly).
    ///
    /// Entry paths are sanitized to prevent writing outside the output
    /// directory (protection against path-traversal / zip-slip attacks).
    #[command(visible_aliases = ["x", "e"])]
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
    /// Prints one line per entry as `SIZE  NAME`, followed by a count. Sizes
    /// are uncompressed bytes where the format records them.
    /// Format detection uses the same extension + magic fallback as extract.
    #[command(visible_aliases = ["l", "ls"])]
    List {
        /// Path of the archive to inspect.
        archive: PathBuf,

        /// Force a format instead of detecting it from the archive extension.
        /// One of: zip, 7z, gz, bz2, zst, tar, rar.
        #[arg(short, long, value_name = "FMT")]
        format: Option<String>,
    },

    /// Update epax to the latest release from GitHub.
    ///
    /// Downloads the latest binary for the current platform, replaces the
    /// running executable in-place, and prints the new version.
    ///
    /// Requires an internet connection. Uses the GitHub Releases API.
    #[command(after_long_help = UPDATE_EXAMPLES)]
    Update {
        /// Only check whether a newer version is available; do not install.
        #[arg(long)]
        check: bool,
    },

    /// Uninstall epax from this system.
    ///
    /// Removes the epax binary. Use --purge to also delete configuration and
    /// cache files. Asks for confirmation unless --force is given.
    #[command(after_long_help = UNINSTALL_EXAMPLES)]
    Uninstall {
        /// Also remove configuration and cache directories.
        #[arg(long)]
        purge: bool,

        /// Skip the confirmation prompt.
        #[arg(long)]
        force: bool,
    },
}

const COMPRESS_EXAMPLES: &str = "\
EXAMPLES:
  epax compress backup.zip ./my-project
  epax compress site.tar.gz ./public index.html
  epax compress release.tar.zst ./bin ./assets -l 19   # high zstd level
  epax compress logs.7z ./logs -v                       # verbose

  # Auto-output: no recognized archive extension → stem + .zip
  epax compress aku.md                   # aku.md → aku.zip
  epax c report.pdf ./assets             # report.zip (report.pdf + assets/)
  epax c notes.txt --format 7z           # notes.7z

  # Single-file bare stream
  epax compress data.csv.gz data.csv     # raw gzip, no tar

NOTES:
  * Output extension picks the format: .zip .7z .tar .tar.gz/.tgz
    .tar.bz2/.tbz2 .tar.zst/.tzst, or bare .gz/.bz2/.zst for one file.
  * RAR output is always rejected (exit code 2).";

const EXTRACT_EXAMPLES: &str = "\
EXAMPLES:
  epax extract backup.zip                  # into current directory
  epax x release.tar.zst -o ./out          # 'x' alias
  epax e photos.rar -o ./photos            # 'e' alias
  epax extract archive.7z -o /tmp/out -v   # verbose

  # Magic-byte detection — extension does not matter
  epax extract myarchive                   # sniffs format from file header
  epax e renamed_zip.dat -o ./unpacked

NOTES:
  * Detection order: --format flag → file extension → magic bytes.
  * Use --format to force a specific format.
    e.g. epax extract blob --format zst -o ./out";

const UPDATE_EXAMPLES: &str = "\
EXAMPLES:
  epax update            # update to latest
  epax update --check    # check only, do not install";

const UNINSTALL_EXAMPLES: &str = "\
EXAMPLES:
  epax uninstall               # interactive confirmation
  epax uninstall --purge       # also remove config/cache files
  epax uninstall --force       # skip confirmation
  epax uninstall --force --purge";
