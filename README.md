# epax

A fast, native, cross-platform archive tool written in Rust. One self-contained
binary that **compresses and extracts** `zip`, `7z`, `gz`, `bz2`, `zst`, and
`tar` archives — and **extracts** `rar` — on **Windows, Linux, and macOS**.

No external CLI tools required at runtime. No `tar`, `gzip`, `7z`, or `unrar`
binaries need to be installed — everything is linked into the single `epax`
executable.

---

## Features

- **Compress & extract** across all common formats with one consistent CLI.
- **Automatic format detection** from the file extension (`.zip`, `.7z`,
  `.tar.gz`, `.tgz`, `.tar.zst`, …), with a `--format` override.
- **Smart stream handling** — directories and multi-file inputs are packed into
  a tar container automatically (`.tar.gz`, `.tar.zst`, …); a single file with a
  bare `.gz`/`.bz2`/`.zst` name is compressed raw, exactly like `gzip`.
- **Path-traversal protection** — every archive entry is sanitized on extraction
  to prevent zip-slip attacks (`../` escapes, absolute paths, drive prefixes are
  rejected).
- **Auto-output mode** — `epax compress aku.md` creates `aku.zip` automatically.
- **Magic-byte detection** — archives without a recognized extension are still
  identified by their file header on extraction.
- **Extract-to-folder by default** — `epax extract backup.zip` creates a `backup/`
  directory and extracts inside it (use `-o` for a custom location).
- **Image squeeze** — re-encode JPEG, PNG and WebP images to any supported
  format, with quality control, batch directory processing and size comparison.
- **Interactive compress** — guided prompts for files, format and options.
- **Recursive directory archiving** with relative path preservation.
- **Unix permission preservation** for `tar` and `zip` (ignored gracefully on
  Windows).
- **Document Text Extraction (`parse`)** — extract text from PDF, DOCX, XLSX, PPTX,
  and images locally without requiring external tools (like LibreOffice or MS Office).
- **In-Process Multi-Engine OCR** — offline OCR using `tesseract` (default), `ocrs` (pure
  Rust), or `paddle` (PaddleOCR via `ocr-rs` using MNN). All model weights are
  embedded directly into the binary at compile time for 100% standalone execution.
- **Document Metadata Inspector (`inspect`)** — inspect page counts, character counts, word counts,
  and text density breakdown with a terminal bar chart.

## Quick start

```bash
# Compress a directory into a zip archive
epax compress backup.zip ./my-project

# Extract it — auto-creates backup/ folder
epax extract backup.zip

# Try interactive mode — no arguments needed
epax compress -i

# Squeeze images to WebP with size comparison
epax squeeze photo.jpg

# See full details
epax --help
```

## Documentation

| Document | Description |
|----------|-------------|
| [`docs/usage.md`](docs/usage.md) | Full command reference with examples for every operation |
| [`docs/architecture.md`](docs/architecture.md) | Module layout, data flow, security, and build system |

## Format support matrix

| Format | Extensions                        | Compress | Extract | Backend                |
|--------|-----------------------------------|:--------:|:-------:|------------------------|
| zip    | `.zip`                            |    ✓     |    ✓    | `zip` (Deflate)        |
| 7z     | `.7z`                             |    ✓     |    ✓    | `sevenz-rust2` (LZMA2) |
| gzip   | `.gz` `.tgz` `.tar.gz`            |    ✓     |    ✓    | `flate2`               |
| bzip2  | `.bz2` `.tbz2` `.tbz` `.tar.bz2`  |    ✓     |    ✓    | `bzip2`                |
| zstd   | `.zst` `.tzst` `.tar.zst`         |    ✓     |    ✓    | `zstd`                 |
| tar    | `.tar`                            |    ✓     |    ✓    | `tar`                  |
| rar    | `.rar`                            |    ✗     |    ✓    | `unrar`                |

> **Why can't epax create RAR archives?**
> RAR is a proprietary format. Its compression algorithm is closed and owned by
> RARLAB — no open-source or native code can legally create `.rar` files; only
> the official non-free WinRAR / `rar` tool can. epax therefore **extracts** RAR
> archives (via the `unrar` library) but refuses to create them, exiting with a
> clear error and exit code `2`.

---

## Installation

### Quick Install (Recommended)

**Linux / macOS:**

```bash
curl -sSL https://raw.githubusercontent.com/podsni/epax/main/scripts/install.sh | bash
```

Install a specific version:

```bash
VERSION=v0.1.0 curl -sSL https://raw.githubusercontent.com/podsni/epax/main/scripts/install.sh | bash
```

**Windows (PowerShell):**

```powershell
iwr -useb https://raw.githubusercontent.com/podsni/epax/main/scripts/install.ps1 | iex
```

Install a specific version:

```powershell
iwr -useb https://raw.githubusercontent.com/podsni/epax/main/scripts/install.ps1 `
  -OutFile install.ps1; .\install.ps1 -Version v0.1.0
```

By default the script installs to `/usr/local/bin` (Linux/macOS) or
`%LOCALAPPDATA%\epax` (Windows) and adds it to your `PATH` automatically.

---

### Manual Download

Download a pre-built binary from the
[**Releases page**](https://github.com/podsni/epax/releases).

| Platform | Architecture | Archive |
|----------|:------------:|---------|
| Linux    | x64          | `epax-x86_64-unknown-linux-gnu.tar.gz` |
| Windows  | x64          | `epax-x86_64-pc-windows-msvc.zip` |

**Linux / macOS manual install:**

```bash
# Download (replace <VERSION> and <TARGET> with your values)
curl -LO https://github.com/podsni/epax/releases/latest/download/epax-x86_64-unknown-linux-gnu.tar.gz

# Extract
tar -xzf epax-x86_64-unknown-linux-gnu.tar.gz

# Install
sudo mv epax-x86_64-unknown-linux-gnu/epax /usr/local/bin/epax
chmod +x /usr/local/bin/epax
```

**Windows manual install:**

1. Download `epax-x86_64-pc-windows-msvc.zip` from the Releases page.
2. Extract the `.zip` to a folder, e.g. `C:\Program Files\epax`.
3. Add that folder to your `PATH` environment variable
   (System Properties → Environment Variables → Path → New).
4. Open a new terminal and run `epax --help`.

---

### Verify Installation

```bash
epax --version
epax --help
```

---

### Update

To update to the latest release, re-run the same install script — it overwrites
the existing binary:

```bash
# Linux / macOS
curl -sSL https://raw.githubusercontent.com/podsni/epax/main/scripts/install.sh | bash

# Windows PowerShell
iwr -useb https://raw.githubusercontent.com/podsni/epax/main/scripts/install.ps1 | iex
```

---

### Uninstall

**Using the uninstall script (recommended):**

Linux / macOS:

```bash
# Interactive (asks for confirmation)
curl -sSL https://raw.githubusercontent.com/podsni/epax/main/scripts/uninstall.sh | bash

# Remove config files too
curl -sSL https://raw.githubusercontent.com/podsni/epax/main/scripts/uninstall.sh | bash -s -- --purge

# No confirmation prompt
curl -sSL https://raw.githubusercontent.com/podsni/epax/main/scripts/uninstall.sh | bash -s -- --force
```

Windows (PowerShell):

```powershell
iwr -useb https://raw.githubusercontent.com/podsni/epax/main/scripts/uninstall.ps1 | iex

# Remove config files too
iwr -useb https://raw.githubusercontent.com/podsni/epax/main/scripts/uninstall.ps1 `
  -OutFile uninstall.ps1; .\uninstall.ps1 -Purge
```

**Manual uninstall:**

Linux / macOS:

```bash
sudo rm /usr/local/bin/epax
# Optionally remove config:
rm -rf ~/.config/epax ~/.local/share/epax
```

Windows:

```
1. Delete  %LOCALAPPDATA%\epax\epax.exe
2. Remove  %LOCALAPPDATA%\epax  from your PATH environment variable
3. Optionally delete  %APPDATA%\epax  for config files
```

---

### Build from source

Requires a [Rust toolchain](https://rustup.rs/) (1.85+ / edition 2024) and a C
compiler (used at build time to compile the vendored `zstd` and `unrar`
sources — `gcc`/`clang` on Unix, MSVC build tools on Windows).

```bash
git clone https://github.com/podsni/epax.git
cd epax
cargo build --release
# binary at target/release/epax  (or target/release/epax.exe on Windows)
```

Install into your Cargo bin directory:

```bash
cargo install --path .
```

**Build without RAR (pure-Rust, compiles everywhere):**

RAR extraction is enabled by default and links the vendored C++ `unrar`
sources. Those require a C++ toolchain and, on Windows, the **MSVC** SDK
(mingw-w64 lacks headers such as `PowrProf.h`, so the `*-pc-windows-gnu` target
cannot build the RAR backend). If you do not need RAR — or you are
cross-compiling with a toolchain that cannot build it — build the all-Rust core
(zip / 7z / gz / bz2 / zst / tar), which compiles everywhere:

```bash
cargo build --release --no-default-features
```

**Windows notes:**

- **Native (recommended):** build on Windows with the MSVC toolchain
  (`x86_64-pc-windows-msvc`) — the full feature set, including RAR, compiles.
- **Cross-compiling from Linux with mingw (`x86_64-pc-windows-gnu`):** build the
  core with `--no-default-features` (the RAR C++ sources need the MSVC SDK).

---

## Releases

Pre-built binaries are published automatically on every version tag push via
GitHub Actions. You can also trigger a release manually:

1. Go to the [**Actions tab**](https://github.com/podsni/epax/actions/workflows/release.yml).
2. Click **"Run workflow"**.
3. Enter the version tag (e.g. `v0.2.0`) — this tag **must already exist** in
   the repository.
4. Click **"Run workflow"** to start the build.

The workflow builds Linux x64 and Windows x64, then creates (or updates) the
GitHub Release for that tag with both archives attached.

**Creating a new release tag:**

```bash
# Bump version in Cargo.toml first, then:
git tag v0.2.0
git push origin v0.2.0
# The release workflow triggers automatically.
```

---

## Usage

```
epax <COMMAND>

Commands:
  compress  Create an archive (format inferred from the output extension)  [alias: c]
  extract   Extract an archive into a directory                            [aliases: x, e]
  list      List the contents of an archive without extracting             [aliases: l, ls]
  squeeze   Compress images — convert between WebP, JPEG, and PNG
  parse     Extract text from document (PDF, DOCX, XLSX, PPTX, image)
  inspect   Show document structural metadata / breakdown                  [alias: info]
  update    Update epax to the latest release from GitHub
  uninstall Uninstall epax from this system
  help      Print this message or the help of the given subcommand(s)
```

### Compress

```bash
epax compress <OUTPUT> <INPUTS>...
```

| Option              | Description                                                                  |
|---------------------|------------------------------------------------------------------------------|
| `-f, --format`      | Override format detection (`zip`, `7z`, `gz`, `bz2`, `zst`, `tar`)          |
| `-l, --level <N>`   | Compression level (clamped to each format's valid range)                     |
| `-v, --verbose`     | Print each entry as it is added                                              |

```bash
# Zip up a directory
epax compress backup.zip ./my-project

# Maximum-compression zstd tarball of multiple inputs
epax compress release.tar.zst ./bin ./assets README.md -l 19

# 7z with LZMA2
epax c docs.7z ./docs

# Compress a single file as a bare gzip stream (-> data.csv.gz)
epax compress data.csv.gz data.csv

# Interactive mode — guided prompts for files, format, and options
epax compress -i
epax compress -i ./docs ./assets            # pre-filled inputs, prompts for rest
epax compress -o out.7z -i ./src            # pre-filled output + inputs
```

### Interactive compress

Run `epax compress -i` (or `epax c -i`) to enter interactive mode — a step-by-step
guide that collects inputs, output format, compression level, and output path
via prompts:

```
── epax interactive compress ──
(press Enter on empty line to finish adding files)

  add file/dir: ./src
  add file/dir: README.md
  add file/dir: 

  inputs (2):
    ./src
    README.md
  output format [zip]: zst
  output path [archive.tar.zst]: release.tar.zst
  compression level (default: format default): 19
  verbose? (y/n) [n]: y

  ─── summary ───
  format:  zst
  output:  release.tar.zst
  inputs:  2
  level:   19
  ───────────────

  proceed? [Y]: Y
  added  ./src/Cargo.toml
  added  ./src/main.rs
  ...
  created release.tar.zst
```

You can also pre-fill inputs and/or output path on the command line and let the
interactive prompts handle the rest:

```bash
epax compress -i ./docs                # pre-fill docs/, prompts for format/level/output
epax compress -o backup.zip -i ./src   # pre-fill output + input, prompts for level etc.
```

### Extract

```bash
epax extract <ARCHIVE> [-o <DIR>]
```

| Option              | Description                                                                  |
|---------------------|------------------------------------------------------------------------------|
| `-o, --output <DIR>`| Destination directory (created if missing; **default: folder named after archive**)|
| `-f, --format`      | Override format detection                                                    |
| `-v, --verbose`     | Print each entry as it is extracted                                          |

```bash
# Extract into a folder named after the archive (default)
epax extract backup.zip                 # → backup/*

# Extract into a specific directory
epax x release.tar.zst -o ./out         # 'x' alias

# Extract using the 'e' alias
epax e photos.rar

# Extract to current directory (explicit -o .)
epax extract archive.7z -o .

# Force format on oddly-named file
epax extract blob --format zst -o ./out
```

### List

```bash
epax list <ARCHIVE>
```

```bash
epax list release.tar.zst
#            6  proj/a.txt
#            5  proj/sub/b.txt
# 2 entries
```

### Compression levels

| Format | Range    | Default |
|--------|----------|---------|
| gzip   | `0`–`9`  | `6`     |
| bzip2  | `1`–`9`  | `6`     |
| zstd   | `1`–`22` | `3`     |
| zip    | `0`–`9`  | `6`     |

Out-of-range values are clamped rather than rejected.

### Squeeze (image compression)

```bash
epax squeeze <INPUTS>... [-o <DIR>] [-f webp|jpeg|png] [-q <N>]
```

Convert JPEG, PNG and WebP images between formats with quality control.
Directories are walked recursively; output preserves the relative structure.

| Option              | Description                                                        |
|---------------------|--------------------------------------------------------------------|
| `-o, --output <DIR>`| Output directory (default: `./squeezed`)                           |
| `-f, --format`      | Output format: `webp`, `jpeg`, or `png` (default: `webp`)          |
| `-q, --quality <N>` | Encoding quality 1–100 (default: `80`; higher = better but larger)  |

```bash
# Convert to WebP (default format)
epax squeeze image.jpg                          # → squeezed/image.webp

# Re-encode as JPEG with quality control
epax squeeze photo.png -f jpeg -q 85            # → squeezed/photo.jpg

# Batch process a whole directory
epax squeeze photos/ -o optimized/ --format webp # → optimized/*.webp

# Lossless PNG re-encode
epax squeeze logo.png -f png -q 100             # → squeezed/logo.png
```

After processing, epax shows a size comparison for each image and a total:

```
  ↓  photo.png        (1.2 MB → 245.6 KB, 80% smaller)
  ↓  screenshot.jpg   (3.1 MB → 856.3 KB, 73% smaller)
  ──
  2 images: 4.3 MB → 1.1 MB  (75% smaller)
  output: squeezed/*.webp
```

**Note:** WebP output uses lossless encoding (quality flag is accepted but
currently ignored for WebP). JPEG output respects `--quality`. PNG is always
lossless.

### Parse (document text extraction)

```bash
epax parse <INPUTS>... [-o <FILE>] [-f text|md|json] [--ocr] [--ocr-engine tesseract|ocrs|paddle]
```

Extract text from PDF, DOCX, XLSX, PPTX, and image files natively and offline. Multiple files can be passed and their outputs will be concatenated.

| Option | Description |
|---|---|
| `-o, --output <FILE>` | Save text output to a file instead of stdout (automatically saved to `parsed.<ext>` for multiple inputs) |
| `-f, --format <FMT>` | Output format: `text` (default), `md` (markdown), or `json` |
| `--ocr` | Enable OCR for scanned pages or images |
| `--ocr-engine <ENGINE>` | Choose OCR engine: `tesseract` (default), `ocrs` (pure Rust ML), or `paddle` (PaddleOCR ML) |
| `--ocr-models-dir <DIR>` | Custom directory to load/download OCR model weights |

#### OCR Model Locations

When using the native engines (`ocrs` or `paddle`), epax automatically checks for or downloads the required model files on disk:
- **Default Location**:
  - **Linux / macOS**: `~/.local/share/epax/models/`
  - **Windows**: `%LOCALAPPDATA%\epax\models\` (e.g., `C:\Users\<User>\AppData\Local\epax\models\`)
- **Custom Location**: Can be specified using the `--ocr-models-dir <DIR>` flag or the `EPAX_OCR_MODELS_DIR` environment variable.
- **Offline / Standalone Fallback**: If the models are not found in the custom or default folders and there is no internet connection to download them, the engine falls back to using the model weights embedded directly inside the compiled binary at build time.


```bash
# Parse a PDF to stdout
epax parse report.pdf

# Parse a Word document and save as Markdown
epax parse document.docx -o document.md -f md

# Parse a spreadsheet (each worksheet is separated into sections)
epax parse data.xlsx

# Run offline OCR on a scanned image using the pure Rust engine
epax parse scanned_page.jpg --ocr --ocr-engine ocrs

# Run offline OCR using the PaddleOCR engine
epax parse scan.png --ocr --ocr-engine paddle
```

### Inspect (document metadata)

```bash
epax inspect <INPUT>
# alias: epax info <INPUT>
```

Print structural metadata for a document, including page counts, character counts, and text-density bar charts per page.

```bash
epax inspect document.pdf
```
```text
file       : document.pdf
format     : PDF (PDFium)
pages      : 3
text items : 485
characters : 3820

per-page breakdown:
  page 1     240 items  ████████████████████████████████████████
  page 2     180 items  ██████████████████████████████
  page 3      65 items  ██████████
```

---

## How stream formats are handled

`gzip`, `bzip2`, and `zstd` are **single-stream** compressors — they compress one
byte stream, not a folder. epax bridges this transparently:

- **Multiple inputs, a directory, or a `.tar.*` name** → inputs are packed into a
  tar archive first, then compressed (`archive.tar.gz`, `archive.tar.zst`, …).
- **A single file with a bare `.gz` / `.bz2` / `.zst` name** → that file's bytes
  are compressed directly, and extraction restores the original name by stripping
  the suffix (`notes.txt.gz` → `notes.txt`), matching the behavior of the standard
  `gzip` tool.

---

## Security

All extraction backends route every archive entry path through a single
`sanitize_entry_path` guard before any file is written. Entries that try to
escape the destination directory — via `../` components, absolute paths, or
Windows drive prefixes — are rejected with an error. This defends against
zip-slip / path-traversal attacks present in maliciously crafted archives.

---

## Development

```bash
cargo build            # debug build
cargo build --release  # optimized build
cargo test             # run the full test suite
cargo clippy           # lints
```

### Test coverage

The suite (`cargo test`) covers:

- **Unit tests** — extension detection for every format (including double
  extensions like `.tar.gz` and short forms like `.tgz`), case-insensitivity,
  compression-suffix stripping, and the path-traversal guard (rejecting `../`
  escapes and absolute paths).
- **Integration tests** (`tests/roundtrip.rs`) — for every writable format, a
  full **compress → extract → byte-for-byte compare** round trip; the bare
  single-file `.gz` path; `list` output; RAR extraction from a committed fixture;
  and verification that creating a RAR fails with exit code `2`.

### Architecture

See [`docs/architecture.md`](docs/architecture.md) for the full architecture
overview — module tree, data flow diagrams, format detection, stream handling,
security model, error handling, feature flags, build system, and memory
characteristics.

Quick reference:

```
src/
  main.rs            entry point: arg parsing, dispatch, exit codes
  cli.rs             clap command/option definitions
  error.rs           EpaxError + exit-code mapping
  format.rs          Format enum, extension detection, tar/suffix logic
  collect.rs         recursive input gathering + archive-name computation
  ops/               compress / extract / list / squeeze dispatch
  backends/          per-format implementations (zip, 7z, streamc, tar, rar)
  util/path.rs       sanitize_entry_path (zip-slip guard)
build.rs             links advapi32 on Windows MSVC when rar feature is on
tests/               integration tests + RAR fixture
scripts/             install / uninstall scripts for Linux, macOS, Windows
.github/workflows/   release.yml — builds all targets, publishes GitHub Release
```

---

## License

MIT
