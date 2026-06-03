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
  format, with quality control and batch directory processing.
- **Recursive directory archiving** with relative path preservation.
- **Unix permission preservation** for `tar` and `zip` (ignored gracefully on
  Windows).

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

The workflow builds Linux x64, macOS arm64, macOS x64, and Windows x64, then
creates (or updates) the GitHub Release for that tag with all four archives
attached.

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

**Note:** WebP output uses lossless encoding (quality flag is accepted but
currently ignored for WebP). JPEG output respects `--quality`. PNG is always
lossless.

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

### Project layout

```
src/
  main.rs            entry point: arg parsing, dispatch, exit codes
  cli.rs             clap command/option definitions
  error.rs           EpaxError + exit-code mapping
  format.rs          Format enum, extension detection, tar/suffix logic
  collect.rs         recursive input gathering + archive-name computation
  ops/               compress / extract / list dispatch by format
  backends/          per-format implementations
    zip.rs  sevenz.rs  streamc.rs  tar.rs  rar.rs
  util/path.rs       sanitize_entry_path (zip-slip guard)
build.rs             links advapi32 on Windows MSVC when the rar feature is on
tests/
  roundtrip.rs       end-to-end integration tests
  fixtures/          sample.rar extraction fixture
scripts/
  install.sh         quick-install for Linux / macOS
  install.ps1        quick-install for Windows PowerShell
  uninstall.sh       uninstall for Linux / macOS
  uninstall.ps1      uninstall for Windows PowerShell
.github/workflows/
  release.yml        builds all targets and publishes GitHub Release on tag push
```

---

## License

MIT
