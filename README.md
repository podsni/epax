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
- **Recursive directory archiving** with relative path preservation.
- **Unix permission preservation** for `tar` and `zip` (ignored gracefully on
  Windows).

## Format support matrix

| Format | Extensions                       | Compress | Extract | Backend             |
|--------|----------------------------------|:--------:|:-------:|---------------------|
| zip    | `.zip`                           |    ✓     |    ✓    | `zip` (Deflate)     |
| 7z     | `.7z`                            |    ✓     |    ✓    | `sevenz-rust2` (LZMA2) |
| gzip   | `.gz` `.tgz` `.tar.gz`           |    ✓     |    ✓    | `flate2`            |
| bzip2  | `.bz2` `.tbz2` `.tbz` `.tar.bz2` |    ✓     |    ✓    | `bzip2`             |
| zstd   | `.zst` `.tzst` `.tar.zst`        |    ✓     |    ✓    | `zstd`              |
| tar    | `.tar`                           |    ✓     |    ✓    | `tar`               |
| rar    | `.rar`                           |    ✗     |    ✓    | `unrar`             |

> **Why can't epax create RAR archives?**
> RAR is a proprietary format. Its compression algorithm is closed and owned by
> RARLAB — no open-source or native code can legally create `.rar` files; only
> the official non-free WinRAR / `rar` tool can. epax therefore **extracts** RAR
> archives (via the `unrar` library) but refuses to create them, exiting with a
> clear error and exit code `2`.

---

## Installation

### From source

Requires a [Rust toolchain](https://rustup.rs/) (1.85+ / edition 2024) and a C
compiler (used at build time to compile the vendored `zstd` and `unrar`
sources — `gcc`/`clang` on Unix, MSVC build tools on Windows).

```bash
git clone git@github.com:podsni/epax.git
cd epax
cargo build --release
# binary at target/release/epax
```

Install into your Cargo bin directory:

```bash
cargo install --path .
```

### Optional: build without RAR

RAR extraction is enabled by default and links the vendored C++ `unrar`
sources. Those require a C++ toolchain and, on Windows, the **MSVC** SDK
(mingw-w64 lacks headers such as `PowrProf.h`, so the `*-pc-windows-gnu` target
cannot build the RAR backend). If you do not need RAR — or you are
cross-compiling with a toolchain that cannot build it — build the all-Rust core
(zip / 7z / gz / bz2 / zst / tar), which compiles everywhere:

```bash
cargo build --release --no-default-features
```

### Windows

- **Native (recommended):** build on Windows with the MSVC toolchain
  (`x86_64-pc-windows-msvc`) — the full feature set, including RAR, compiles.
- **Cross-compiling from Linux with mingw (`x86_64-pc-windows-gnu`):** build the
  core with `--no-default-features` (the RAR C++ sources need the MSVC SDK).

---

## Usage

```
epax <COMMAND>

Commands:
  compress  Create an archive (format inferred from the output extension)  [alias: c]
  extract   Extract an archive into a directory                            [alias: x]
  list      List the contents of an archive without extracting             [aliases: l, ls]
  help      Print this message or the help of the given subcommand(s)
```

### Compress

```bash
epax compress <OUTPUT> <INPUTS>...
```

| Option              | Description                                              |
|---------------------|----------------------------------------------------------|
| `-f, --format`      | Override format detection (`zip`, `7z`, `gz`, `bz2`, `zst`, `tar`) |
| `-l, --level <N>`   | Compression level (clamped to each format's valid range) |
| `-v, --verbose`     | Print each entry as it is added                          |

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

| Option              | Description                                              |
|---------------------|----------------------------------------------------------|
| `-o, --output <DIR>`| Destination directory (created if missing; default `.`)  |
| `-f, --format`      | Override format detection                                |
| `-v, --verbose`     | Print each entry as it is extracted                      |

```bash
# Extract into the current directory
epax extract backup.zip

# Extract into a specific directory
epax x release.tar.zst -o ./out

# Extract a RAR archive
epax extract photos.rar -o ./photos
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
tests/
  roundtrip.rs       end-to-end integration tests
  fixtures/          sample.rar extraction fixture
```

---

## License

MIT
