# epax

Cross-platform archive CLI (zip/7z/gz/bz2/zst/tar/rar). Single self-contained
Rust binary — no external tools needed at runtime.

## Setup

```bash
cargo build                    # debug
cargo build --release          # optimized
cargo build --no-default-features  # pure-Rust core (no RAR, no C++ toolchain)
```

## Test

```bash
cargo test                     # all
cargo test --test roundtrip    # integration only
cargo test <name>              # single (e.g. roundtrip_zip)
cargo clippy
```

## Release

1. Bump version in `Cargo.toml`
2. `git tag vX.Y.Z && git push origin vX.Y.Z`
3. CI builds Linux x64 + Windows x64 MSVC binaries, publishes GitHub Release

## Key architecture

- **Format detection**: `--format` flag → file extension (longest suffix wins) → magic bytes
- **Stream formats** (gz/bz2/zst): multi-file or directory = tar container; bare single file = raw stream
- **Extract-to-folder**: default output dir = archive name minus all suffixes
- **Auto-output**: no archive extension on first arg → treat as input, auto-create `<stem>.zip`
- **RAR**: feature-gated (default on), extract-only. Exit code 2 if compress attempted.
- **Interactive**: `epax compress -i` or omit OUTPUT. Stdin prompts with sensible defaults.
- **Squeeze**: re-encode images, shows per-file size comparison.
- **Zip-slip guard**: `sanitize_entry_path()` in `src/util/path.rs` — every extract backend routes through it.

## Code conventions

- `thiserror` for error types, `EpaxError::Backend(String)` for backend failures
- Backend functions: `compress(output, entries, level, verbose)` and `extract(archive, dest, verbose)`
- `clamp_level(level, min, max, default)` in `src/backends/mod.rs` for level validation
- `collect_inputs()` returns `Vec<Entry>` with `{path, arcname}` (forward-slash archive names)
- Format mapping in `src/format.rs`: `detect_from_path`, `detect_from_magic`, `name_implies_tar`, suffix helpers
- `cfg(feature = "rar")` gates all RAR code including `src/backends/rar.rs`
- `build.rs` links `advapi32` on Windows MSVC when rar feature is enabled

## PR

- Title: `feat:`, `fix:`, `docs:`, `chore:` prefix
- Tag push triggers release — make sure version in `Cargo.toml` is bumped
- Include `Co-Authored-By: Claude` if applicable
