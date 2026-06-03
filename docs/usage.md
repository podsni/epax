# Usage Guide

## Global options

```
epax <COMMAND> [OPTIONS] [ARGS]
```

| Command      | Alias | Description                                         |
|--------------|-------|-----------------------------------------------------|
| `compress`   | `c`   | Create an archive from files and directories        |
| `extract`    | `x`   | Extract an archive into a directory                 |
| `list`       | `l`   | List archive contents without extracting            |
| `squeeze`    |       | Re-encode images (WebP/JPEG/PNG conversion)         |
| `update`     |       | Update epax to the latest release from GitHub       |
| `uninstall`  |       | Remove epax from this system                        |

---

## Compress

```
epax compress [OUTPUT] [INPUTS]... [OPTIONS]
epax c [OUTPUT] [INPUTS]... [OPTIONS]
```

### Options

| Option                | Description                                         |
|-----------------------|-----------------------------------------------------|
| `-f, --format <FMT>`  | Force format: `zip`, `7z`, `gz`, `bz2`, `zst`, `tar` |
| `-l, --level <N>`     | Compression level (clamped per format; see levels)  |
| `-v, --verbose`       | Print each entry as it is added to the archive      |
| `-i, --interactive`   | Interactive mode — guided step-by-step prompts      |

### How output is determined

1. If `OUTPUT` has a recognized archive extension (`.zip`, `.7z`, `.tar.gz`,
   etc.), it is used as-is.
2. If `OUTPUT` has no recognized extension and `--format` is given, the
   output name is auto-generated: `<stem>.<format-ext>`. The `OUTPUT`
   argument becomes an input.
3. If `OUTPUT` has no recognized extension and no `--format`:
   auto-output mode — `<stem-of-first-input>.zip`.
4. If `OUTPUT` is omitted: interactive mode is entered.

### Format auto-selection by extension

| Extension                  | Format  | Container |
|----------------------------|---------|-----------|
| `.zip`                     | zip     | zip       |
| `.7z`                      | 7z      | 7z        |
| `.tar`                     | tar     | tar       |
| `.tar.gz` `.tgz`           | gzip    | tar       |
| `.tar.bz2` `.tbz2` `.tbz` | bzip2   | tar       |
| `.tar.zst` `.tzst`         | zstd    | tar       |
| `.gz`                      | gzip    | none/bare |
| `.bz2`                     | bzip2   | none/bare |
| `.zst`                     | zstd    | none/bare |
| `.rar`                     | rar     | —         |

### Examples

**Basic directory compression:**

```bash
epax compress backup.zip ./my-project
```

**Multiple inputs with zstd at max level:**

```bash
epax compress release.tar.zst ./bin ./assets README.md -l 19
```

**Auto-output — no extension on output name:**

```bash
epax compress aku.md                    # → aku.zip
epax c report.pdf                       # → report.zip
epax compress notes.txt --format 7z     # → notes.7z
```

**7z with LZMA2 (default):**

```bash
epax c docs.7z ./docs
```

**Bare single-file gzip (no tar wrapper):**

```bash
epax compress data.csv.gz data.csv
```

**Verbose mode — see every entry:**

```bash
epax c archive.zip ./project -v
# added  ./project/src/main.rs
# added  ./project/src/lib.rs
# added  ./project/README.md
```

**Interactive mode:**

```bash
epax compress -i
epax compress -i ./docs ./assets          # pre-filled inputs
epax compress -o out.7z -i ./src          # pre-filled output + inputs
```

See the [Interactive compress](#interactive-compress) section for the full
interactive session walkthrough.

### Compression levels

| Format | Range    | Default | Notes                                      |
|--------|----------|---------|--------------------------------------------|
| zip    | 0–9      | 6       | 0 = store, 6 = default deflate, 9 = max    |
| 7z     | —        | LZMA2   | Fixed LZMA2 (no level control from CLI)    |
| gzip   | 0–9      | 6       | 0 = none, 1 = fastest, 9 = best            |
| bzip2  | 1–9      | 6       | 1 = fast, 9 = small but slow               |
| zstd   | 1–22     | 3       | 3 = fast/decent, 19 = slow/small, 22 = ultra|
| tar    | —        | —       | No compression (just archive)              |

Out-of-range values are clamped to the nearest valid value.

---

## Extract

```
epax extract <ARCHIVE> [OPTIONS]
epax x <ARCHIVE> [OPTIONS]
epax e <ARCHIVE> [OPTIONS]
```

### Options

| Option                | Description                                           |
|-----------------------|-------------------------------------------------------|
| `-o, --output <DIR>`  | Destination directory (default: folder named after archive) |
| `-f, --format <FMT>`  | Force format instead of auto-detecting                |
| `-v, --verbose`       | Print each entry as it is extracted                   |

### Default output directory

When `-o` is not given, epax creates a folder named after the archive
with all archive suffixes stripped:

| Archive                    | Default output dir |
|----------------------------|--------------------|
| `backup.zip`               | `backup/`          |
| `release.tar.zst`          | `release/`         |
| `photos.rar`               | `photos/`          |
| `data (2024).tar.gz`       | `data (2024)/`     |

### Format detection order

1. `--format` flag — exact match (skips file inspection)
2. File extension — longest suffix match (`.tar.gz` before `.gz`)
3. Magic bytes — reads first 8 bytes of the file

### Examples

**Default — extracts into folder named after archive:**

```bash
epax extract backup.zip               # → backup/*
```

**Explicit output directory:**

```bash
epax x release.tar.zst -o ./out
```

**Using the short alias:**

```bash
epax e photos.rar                     # → photos/*
```

**Extract to current directory:**

```bash
epax extract archive.7z -o .
```

**Force format on a file without a recognized extension:**

```bash
epax extract blob --format zst -o ./out
```

**Verbose extraction:**

```bash
epax extract archive.zip -v
# backup/src/main.rs
# backup/README.md
# 2 entries
```

---

## List

```
epax list <ARCHIVE> [OPTIONS]
epax l <ARCHIVE> [OPTIONS]
epax ls <ARCHIVE> [OPTIONS]
```

### Options

| Option                | Description                           |
|-----------------------|---------------------------------------|
| `-f, --format <FMT>`  | Force format instead of auto-detecting |

### Examples

```bash
epax list release.tar.zst
#            6  proj/a.txt
#            5  proj/sub/b.txt
# 2 entries
```

```bash
epax l archive.zip
#         1024  docs/manual.pdf
#          843  docs/readme.txt
# 2 entries
```

---

## Squeeze (image compression)

```
epax squeeze <INPUTS>... [OPTIONS]
```

### Options

| Option                   | Description                                          |
|--------------------------|------------------------------------------------------|
| `-o, --output <DIR>`     | Output directory (default: `./squeezed`)             |
| `-f, --format <FMT>`     | Output format: `webp`, `jpeg`, `png` (default: `webp`) |
| `-q, --quality <N>`      | Quality 1–100 (default: `80`; higher = better but larger) |

### Size comparison output

After processing, epax shows per-image and total size comparison:

```
  ↓  photo.png        (1.2 MB → 245.6 KB, 80% smaller)
  ↓  screenshot.jpg   (3.1 MB → 856.3 KB, 73% smaller)
  ──
  2 images: 4.3 MB → 1.1 MB  (75% smaller)
  output: squeezed/*.webp
```

Symbols: `↓` = smaller, `=` = same size. Only possible size increase would
happen when converting from a lossy format to a larger lossless format
(e.g. JPEG → PNG at high quality).

### Format notes

| Format | Quality control | Behavior                                         |
|--------|:---------------:|---------------------------------------------------|
| WebP   | Ignored         | Lossless encoding only (quality flag accepted, not applied) |
| JPEG   | Respected       | `--quality` controls compression level            |
| PNG    | Ignored         | Always lossless                                   |

### Examples

**Single file to WebP:**

```bash
epax squeeze image.jpg                    # → squeezed/image.webp
```

**JPEG with quality control:**

```bash
epax squeeze photo.png -f jpeg -q 85      # → squeezed/photo.jpg
```

**Batch process directory:**

```bash
epax squeeze photos/ -o optimized/ --format webp
```

**Lossless PNG re-encode:**

```bash
epax squeeze logo.png -f png -q 100       # → squeezed/logo.png
```

**High-quality JPEG from a directory:**

```bash
epax squeeze vacation/ -o jpegs/ -f jpeg -q 90
```

---

## Interactive compress

Run `epax compress -i` to enter interactive mode. The tool prompts for
each parameter step by step with sensible defaults.

### Full session walkthrough

```
── epax interactive compress ──
(press Enter on empty line to finish adding files)

  add file/dir: ./src
  add file/dir: README.md
  add file/dir:              ← empty line = done

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
  added  ./src/main.rs
  added  ./src/cli.rs
  ...
  created release.tar.zst
```

### Pre-filling arguments

CLI arguments populate the interactive defaults:

```bash
epax compress -i ./docs                # docs/ pre-filled, prompts for format/level/output
epax compress -o backup.zip -i ./src   # output + input pre-filled, prompts for level
epax compress backup.zip ./src -i      # everything pre-filled, just confirm
```

### Interactive behavior

- Empty line at file prompt: stops collecting files
- Format prompt defaults to `zip`
- Output path auto-generated from first input's file stem + format ext
- Compression level prompt: `auto` = format default
- Verbose prompt defaults to `n`
- Proceed prompt defaults to `Y`
- Any non-"yes" response to proceed aborts

---

## Update

```
epax update [--check]
```

| Option   | Description                                  |
|----------|----------------------------------------------|
| `--check`| Only check for newer version; do not install |

Fetches the latest release tag from GitHub API and compares with the
current version. On confirmation, re-runs the install script.

```bash
epax update                  # check + install if newer
epax update --check          # check only
```

---

## Uninstall

```
epax uninstall [--purge] [--force]
```

| Option   | Description                                         |
|----------|-----------------------------------------------------|
| `--purge`| Also remove config and data directories             |
| `--force`| Skip confirmation prompt                            |

```bash
epax uninstall               # interactive confirmation, remove binary
epax uninstall --purge       # + remove ~/.config/epax and ~/.local/share/epax
epax uninstall --force       # skip confirmation
```

---

## Exit codes

| Code | Meaning                       |
|:----:|-------------------------------|
| 0    | Success                       |
| 1    | Error (IO, format, input)     |
| 2    | Cannot create this format     |
