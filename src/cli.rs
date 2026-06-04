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
///             Extracts into a folder named after the archive by default.
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
  # Compress
  epax compress backup.zip ./my-project
  epax compress release.tar.zst ./bin ./assets
  epax c docs.7z ./docs                         # 'c' alias

  # Compress — auto output name
  epax compress aku.md                          # → aku.zip
  epax c report.pdf                             # → report.zip
  epax c notes.txt --format 7z                  # → notes.7z

  # Compress — interactive mode
  epax compress -i                              # guided step-by-step

  # Extract — auto creates folder named after archive
  epax extract backup.zip                       # → backup/* (no -o needed)
  epax x photos.rar -o ./out                    # explicit output dir
  epax e archive.7z                             # → archive/*

  # Parse documents (PDF, DOCX, XLSX, PPTX, images)
  epax parse report.pdf                         # → text to stdout
  epax parse thesis.pdf -o thesis.txt           # → saved to file
  epax parse scan.pdf --ocr                     # enable OCR for scanned pages
  epax parse report.pdf --format json           # structured JSON output

  # Inspect document metadata
  epax inspect report.pdf                       # page count + text items
  epax info   report.pdf                        # alias

  # Squeeze images
  epax squeeze image.jpg                        # → squeezed/image.webp
  epax squeeze photo.png -o optimized/          # → optimized/photo.webp
  epax squeeze *.jpg --quality 90               # batch, high quality

  # Single-file stream
  epax compress data.csv.gz data.csv
  epax extract data.csv.gz                      # → data.csv (bare stream)

SUPPORTED FORMATS:
  zip  7z  gz  bz2  zst  tar      compress + extract
  rar                             extract only (proprietary format)

DOCUMENT PARSING (epax parse / epax inspect):
  pdf  docx  xlsx  pptx           text extraction via LiteParse
  jpg  png  webp                  OCR-capable image parsing

MANAGE:
  epax update                   # update to the latest release
  epax uninstall                # uninstall epax from this system

TIP: run `epax help <command>` for full details.";


#[derive(Subcommand, Debug)]
pub enum Command {
    /// Create an archive from files and/or directories.
    ///
    /// AUTO OUTPUT NAME
    ///   If OUTPUT has no recognized archive extension and --format is not
    ///   given, OUTPUT is treated as an additional input and the output name
    ///   is generated automatically: <stem-of-first-input>.zip
    ///
    /// INTERACTIVE MODE
    ///   Run with -i (or omit OUTPUT entirely) to enter interactive mode,
    ///   which guides you through choosing files, format, compression level
    ///   and output path step by step.
    #[command(visible_alias = "c")]
    #[command(after_long_help = COMPRESS_EXAMPLES)]
    Compress {
        /// Output path. If omitted (or with -i), enters interactive mode.
        output: Option<PathBuf>,
        #[arg()]
        inputs: Vec<PathBuf>,
        #[arg(short, long, value_name = "FMT")]
        format: Option<String>,
        #[arg(short, long, value_name = "N")]
        level: Option<i32>,
        #[arg(short, long)]
        verbose: bool,
        /// Interactive mode — guide through compression step by step.
        #[arg(short, long)]
        interactive: bool,
    },

    /// Extract an archive into a directory.
    ///
    /// By default, extracts into a folder named after the archive (without its
    /// extension). Use -o to extract to a specific location.
    #[command(visible_aliases = ["x", "e"])]
    #[command(after_long_help = EXTRACT_EXAMPLES)]
    Extract {
        /// Path of the archive to extract.
        archive: PathBuf,

        /// Directory to extract into. Default: a folder named after the archive.
        #[arg(short, long, value_name = "DIR")]
        output: Option<PathBuf>,

        /// Force a format instead of detecting it.
        #[arg(short, long, value_name = "FMT")]
        format: Option<String>,

        /// Print the name of each entry as it is extracted.
        #[arg(short, long)]
        verbose: bool,
    },

    /// List the contents of an archive without extracting it.
    #[command(visible_aliases = ["l", "ls"])]
    List {
        archive: PathBuf,
        #[arg(short, long, value_name = "FMT")]
        format: Option<String>,
    },

    /// Compress images — convert to WebP (default), JPEG, or PNG.
    ///
    /// Reads JPEG, PNG, and WebP images and re-encodes them at the specified
    /// quality. Output goes into a folder named 'squeezed' (or any -o dir).
    /// Directories are walked recursively.
    #[command(after_long_help = SQUEEZE_EXAMPLES)]
    Squeeze {
        /// Image files or directories to process.
        #[arg(required = true)]
        inputs: Vec<PathBuf>,

        /// Output directory (default: ./squeezed).
        #[arg(short, long, default_value = "squeezed", value_name = "DIR")]
        output: PathBuf,

        /// Output image format: webp, jpeg, png (default: webp).
        #[arg(short, long, default_value = "webp", value_name = "FMT")]
        format: String,

        /// Encoding quality 1-100 (default: 80). Higher = better/larger.
        #[arg(short, long, default_value_t = 80, value_name = "N")]
        quality: u8,
    },

    /// Update epax to the latest release from GitHub.
    #[command(after_long_help = UPDATE_EXAMPLES)]
    Update {
        /// Only check whether a newer version is available; do not install.
        #[arg(long)]
        check: bool,
    },

    /// Uninstall epax from this system.
    #[command(after_long_help = UNINSTALL_EXAMPLES)]
    Uninstall {
        #[arg(long)]
        purge: bool,
        #[arg(long)]
        force: bool,
    },

    /// Extract text from a document — PDF, DOCX, XLSX, PPTX, or image.
    ///
    /// Uses LiteParse for local, spatial PDF text extraction — no cloud,
    /// no external tools required at runtime.
    ///
    /// Multiple input files are supported; their text is concatenated.
    /// Use --ocr to enable Tesseract for scanned / image-heavy pages.
    #[cfg(feature = "parse")]
    #[command(after_long_help = PARSE_EXAMPLES)]
    Parse {
        /// Document files to parse (PDF, DOCX, XLSX, PPTX, JPG, PNG, …).
        #[arg(required = true)]
        inputs: Vec<PathBuf>,

        /// Write output to this file instead of stdout.
        /// For multiple inputs without -o, output is auto-saved to parsed.<ext>.
        #[arg(short, long, value_name = "FILE")]
        output: Option<PathBuf>,

        /// Output format: text (default), md / markdown, or json.
        #[arg(short, long, default_value = "text", value_name = "FMT")]
        format: String,

        /// Enable OCR for scanned pages (requires Tesseract on PATH).
        #[arg(long)]
        ocr: bool,

        /// OCR engine: tesseract (default), ocrs (native Rust), or paddle (native Rust PaddleOCR).
        #[arg(long, default_value = "tesseract", value_name = "ENGINE")]
        ocr_engine: String,
    },

    /// Show document metadata: page count, text-item counts per page.
    ///
    /// Reads the document with LiteParse and reports structural information
    /// without printing the full extracted text.
    #[cfg(feature = "parse")]
    #[command(visible_alias = "info", after_long_help = INSPECT_EXAMPLES)]
    Inspect {
        /// Document file to inspect.
        input: PathBuf,
    },
}


const COMPRESS_EXAMPLES: &str = "\
EXAMPLES:
  epax compress backup.zip ./my-project
  epax compress site.tar.gz ./public index.html
  epax c release.tar.zst ./bin -l 19
  epax compress notes.txt --format 7z          # auto-output: notes.7z

  # Interactive mode
  epax compress -i                            # fully interactive
  epax compress -i ./docs ./assets            # pre-fill inputs, interactive prompts for rest
  epax compress -o output.zip -i ./src        # pre-fill output + inputs, prompts for rest

  # Single-file bare stream
  epax compress data.csv.gz data.csv";

const EXTRACT_EXAMPLES: &str = "\
EXAMPLES:
  epax extract backup.zip                      # → backup/ folder
  epax x photos.rar -o ./photos                # explicit output dir
  epax e archive.7z                            # → archive/ folder

NOTES:
  * Default output is a folder named after the archive (extension stripped).
  * Use -o <DIR> to extract anywhere.
  * Detection: --format flag → extension → magic bytes.";

const SQUEEZE_EXAMPLES: &str = "\
EXAMPLES:
  epax squeeze image.jpg                       # → squeezed/image.webp
  epax squeeze photo.png -o optimized/         # → optimized/photo.webp
  epax squeeze *.jpg --quality 90              # batch, high quality
  epax squeeze photos/ --format jpeg           # re-encode folder as JPEG
  epax squeeze logo.png --format png --quality 100  # lossless PNG";

const UPDATE_EXAMPLES: &str = "\
EXAMPLES:
  epax update              # update to latest release
  epax update --check      # check only, do not install";

const UNINSTALL_EXAMPLES: &str = "\
EXAMPLES:
  epax uninstall                # interactive confirmation
  epax uninstall --purge        # also remove config/cache
  epax uninstall --force        # skip confirmation";

#[cfg(feature = "parse")]
const PARSE_EXAMPLES: &str = "\
EXAMPLES:
  epax parse report.pdf                       # text to stdout
  epax parse thesis.pdf -o thesis.txt         # save to file
  epax parse scan.pdf --ocr                   # OCR for scanned pages
  epax parse report.pdf --format json         # JSON output
  epax parse a.pdf b.docx c.xlsx              # multiple files concatenated

SUPPORTED DOCUMENT TYPES:
  pdf   — spatial text extraction via PDFium
  docx  — Microsoft Word
  xlsx  — Microsoft Excel
  pptx  — Microsoft PowerPoint
  jpg, png, webp — image OCR (use --ocr)";

#[cfg(feature = "parse")]
const INSPECT_EXAMPLES: &str = "\
EXAMPLES:
  epax inspect report.pdf     # show page count + text items per page
  epax info   report.pdf      # same, using alias";

