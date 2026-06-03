# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Commands

```bash
# Build
cargo build                    # debug
cargo build --release          # optimized

# Test
cargo test                     # all tests
cargo test --test roundtrip    # integration tests only
cargo test <test_name>         # single test (e.g. roundtrip_zip)

# Lint
cargo clippy

# Build without RAR (pure-Rust, no C++ toolchain needed)
cargo build --no-default-features

# Cross-compile for Windows from Linux
cargo build --release --target x86_64-pc-windows-gnu --no-default-features

# Release (CI will build binaries after tag push)
# 1. bump version in Cargo.toml
# 2. git tag vX.Y.Z && git push origin vX.Y.Z
```

## Architecture

Single-binary CLI tool. All backends statically linked. Vendored C sources
(zstd, bzip2, unrar) compile from source during `cargo build`.

### Module tree

```
src/
  main.rs         dispatch+exit codes, update/uninstall subcommands
  cli.rs          clap derive: Cli, Command enum, after_long_help examples
  error.rs        EpaxError enum (thiserror), exit_code() mapping
  format.rs       Format enum, detect_from_path/magic, suffix helpers
  collect.rs      walkdir expansion, Entry {path, arcname}
  ops/            dispatch per operation
    compress.rs   resolve_output, run (dispatch), run_interactive
    extract.rs    resolve_format, derive_output_dir
    list.rs       format dispatch
    squeeze.rs    image re-encoding via `image` crate
  backends/       per-format read/write
    zip.rs        Deflate via `zip` crate
    sevenz.rs     LZMA2 via `sevenz-rust2`
    streamc.rs    gz/bz2/zst encode/decode + tar wrap/unwrap dispatch
    tar.rs        tar via `tar` crate
    rar.rs        extract+list via `unrar` (cfg-gated)
  util/path.rs    sanitize_entry_path — zip-slip guard
```

### Key invariants

- **Format detection chain**: `--format` flag → extension (longest suffix
  first, e.g. `.tar.gz` before `.gz`) → magic bytes (first 8 bytes, extract
  only fallback)
- **Stream formats** (gz/bz2/zst): multi-file or directory or `.tar.*` name →
  tar container; single bare stream file → raw compression. Determined by
  `name_implies_tar()` in `format.rs`.
- **Extract-to-folder**: default output dir = `strip_archive_suffix(archive_name)`.
  All extraction routes through `sanitize_entry_path()` in `util/path.rs`.
- **Auto-output compress**: when `OUTPUT` has no recognized archive extension,
  `resolve_output()` treats it as input + auto-generates `<stem>.zip`.
- **RAR**: feature-gated (default on), extract-only. `CompressNotSupported`
  returns exit code 2. `build.rs` links `advapi32` on Windows MSVC when
  rar feature is enabled.
- **Interactive mode**: `op is_none || interactive` → `run_interactive()`.
  Stdin prompts for files, format, level, output. Pre-fills from CLI args.
- **Squeeze size comparison**: `process_image()` returns compressed size.
  `run()` aggregates per-file and total size with `human_size()` and pct.

### CLI surface (clap derive)

- Commands: `Compress` (alias c), `Extract` (aliases x, e), `List` (aliases l, ls),
  `Squeeze`, `Update`, `Uninstall`
- `after_long_help` constants per command (e.g. `COMPRESS_EXAMPLES`)
- Compress: `output: Option<PathBuf>` (None → interactive), `-i` flag
- Extract: `output: Option<PathBuf>` (None → derive from archive name)
- `level: Option<i32>` passed through, clamped per backend by `clamp_level()`

### Error handling

All ops return `Result<T, EpaxError>`. `main()` maps to ExitCode: 0 success,
1 generic error, 2 CompressNotSupported (RAR). `EpaxError` is thiserror
derive. Backend errors wrapped as `EpaxError::Backend(String)`.

### Test structure

Integration tests in `tests/roundtrip.rs` drive compiled binary via
`assert_cmd::Command::cargo_bin`. Roundtrips for every writable format
(compress → extract → byte-for-byte verify). RAR test uses committed
fixture `tests/fixtures/sample.rar`. `tempfile::TempDir` for isolation.
