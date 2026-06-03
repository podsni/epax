# Architecture

## Overview

epax is a single self-contained Rust binary. All archive backends (zip, 7z,
gz, bz2, zst, tar, rar) are statically linked at compile time — no external
CLI tools required at runtime. The vendored C sources for `zstd` and `unrar`
are compiled from source during `cargo build`.

---

## Module tree

```
src/
  main.rs         →  arg dispatch, exit codes, update/uninstall
  cli.rs          →  clap derive definitions
  error.rs        →  EpaxError + exit-code mapping
  format.rs       →  Format enum, extension/magic-byte detection
  collect.rs      →  recursive input gathering + archive-name logic
  ops/
    compress.rs   →  compress dispatch + interactive mode
    extract.rs    →  extract dispatch + output-dir derivation
    list.rs       →  list dispatch
    squeeze.rs    →  image re-encoding (WebP/JPEG/PNG)
  backends/
    zip.rs        →  zip read/write via `zip` crate
    sevenz.rs     →  7z read/write via `sevenz-rust2`
    streamc.rs    →  gz/bz2/zst encode/decode + tar wrap/unwrap
    tar.rs        →  tar read/write via `tar` crate
    rar.rs        →  rar extract/list via `unrar` (optional, cfg-gated)
  util/
    path.rs       →  sanitize_entry_path() — zip-slip guard
```

---

## Data flow

### Compress

```
CLI args → ops::compress::run()
            → resolve_output()  — detect format from ext / --format
            → collect_inputs()  — walkdir recursive expansion
            → backend::compress()
                zip/7z/tar:  write entries directly
                gz/bz2/zst:  if multi-file OR .tar.* name:
                                write tarball → stream encoder
                              else:
                                write raw stream encoder
            → finalize archive (drop encoder → flush trailer)
```

### Extract

```
CLI args → ops::extract::run()
            → resolve_format()  — try --format, then ext, then magic bytes
            → derive_output_dir()  — strip all archive suffixes from name
            → backend::extract()
                zip/7z/tar/rar:  iterate entries → sanitize → write files
                gz/bz2/zst:  probe stream content for tar magic
                                if tar:  decode stream → untar → sanitize → write
                                else:  decode stream → single output file
```

### Squeeze

```
CLI args → ops::squeeze::run()
            → create output dir
            → for each input:
                if dir: walkdir, process each supported image
                if file: process single image
            → process_image()
                decode via `image::open()`
                re-encode in target format
                report original vs compressed size
```

---

## Format detection

Two-tier detection chain:

1. **Extension-based** (`Format::detect_from_path`): match file extension
   against known suffixes. Double extensions checked first (`.tar.gz`,
   `.tar.bz2`, `.tar.zst`, `.tgz`, `.tzst`) before single extensions.
   Case-insensitive.

2. **Magic-byte** (`Format::detect_from_magic`): read first 8 bytes, match
   against known signatures:

   | Signature            | Format |
   |----------------------|--------|
   | `PK\x03\x04`         | zip    |
   | `7z\xbc\xaf\x27\x1c` | 7z     |
   | `\x1f\x8b`           | gzip   |
   | `BZh`                | bzip2  |
   | `\x28\xb5\x2f\xfd`   | zstd   |
   | `Rar!\x1a\x07`       | rar    |
   | `ustar` at byte 257  | tar    |

Magic-byte detection is only used as a fallback on extract (when the file
has no recognized extension).

---

## Stream format handling

`gzip`, `bzip2`, and `zstd` are single-stream compressors — they compress
one byte stream, not a directory structure. epax bridges this transparently:

- **Multi-file or directory input** → inputs packed into a tar archive, then
  the tar bytes are streamed through the compressor. On extraction, the
  decompressed stream is probed: if it starts with tar magic bytes, the
  tar entries are extracted; otherwise the data is written as a single file.
- **Single file with bare stream extension** (`.gz`, `.bz2`, `.zst`) →
  raw compression of file bytes, no tar wrapper. Extraction restores the
  original file name by stripping the compression suffix.

The `name_implies_tar()` function determines tar-wrapping based on the
output path: `.tar.gz`, `.tgz`, `.tar.bz2`, `.tbz2`, `.tbz`, `.tar.zst`,
`.tzst` all imply tar mode.

---

## Security

All extraction backends call `sanitize_entry_path()` in `util/path.rs`
before writing any file. The function normalizes the archive entry path
through `Path::components()` and:

- Rejects entries starting with `..` (depth < 0)
- Rejects absolute paths (`Component::RootDir`)
- Rejects Windows drive prefixes (`Component::Prefix`)
- Allows interior `..` components (e.g. `a/../b/c`) as long as they don't
  escape the output root

This defends against zip-slip / path-traversal attacks.

---

## Error handling & exit codes

All operations return `Result<T, EpaxError>` (thiserror enum). `main()`
matches the result and maps to exit codes:

| Exit code | Meaning                     | Example                                   |
|-----------|-----------------------------|-------------------------------------------|
| 0         | Success                     | —                                         |
| 1         | Generic error               | IO error, bad format, missing input       |
| 2         | Compress not supported      | `epax c out.rar ...`                      |

The `EpaxError` variants cover: `Io`, `UnknownFormat`, `BadFormatName`,
`CompressNotSupported`, `RarUnavailable` (cfg-gated), `UnsafePath`,
`NoInputs`, `MissingInput`, `Backend`.

---

## Feature flags

| Feature | Default | Description                          |
|---------|:-------:|--------------------------------------|
| `rar`   | on      | RAR extraction via `unrar` C++ lib   |

When `rar` is off (`--no-default-features`), epax compiles entirely with
Rust code — no C++ toolchain needed. All other formats (zip/7z/gz/bz2/zst/tar)
are pure Rust and always included.

---

## Build system

`build.rs` links `advapi32` on Windows MSVC when the `rar` feature is
enabled (required by unrar-sys for registry and process APIs). On other
platforms the build script is a no-op.

Dependencies requiring C compilation: `zstd` (vendored libzstd), `bzip2`
(vendored libbz2), `unrar` (vendored C++ sources, optional).

---

## Memory and performance characteristics

- **Compression**: all formats process entries sequentially. Entire input
  file is read into memory for zip/7z single-file entries. For stream
  formats (gz/bz2/zst), data is piped through encoder buffers — memory
  usage stays proportional to largest input file plus encoder window.
- **Extraction**: entries are streamed and written incrementally. Maximum
  memory usage is bounded by the largest single entry plus decompressor
  window.
- **Squeeze** (image): the `image` crate decodes the entire image into a
  pixel buffer (width × height × 4 bytes for RGBA). A 4000×3000 JPEG
  uses ~48 MB of pixel buffer memory.
- **Parallelism**: all operations are single-threaded. Compression level
  affects CPU usage (higher levels = more CPU, less I/O).
