# Parse & Inspect

**Requires**: `parse` feature (enabled by default).  
**Backend**: [LiteParse 2.x](https://github.com/run-llama/liteparse) — fast,
local PDF/document parsing via PDFium. No internet, no cloud.

---

## `epax parse`

```
epax parse <INPUTS>... [OPTIONS]
```

Extract text from one or more documents. Output goes to **stdout** by default;
use `-o` to save to a file. When multiple inputs are given without `-o`, output
is auto-saved to `parsed.<ext>` in the current directory.

### Options

| Option                    | Description                                                   |
|---------------------------|---------------------------------------------------------------|
| `-o, --output <FILE>`     | Write output to this file instead of stdout                   |
| `-f, --format <FMT>`      | Output format: `text` (default), `md` / `markdown`, or `json`|
| `--ocr`                   | Enable Tesseract OCR for scanned / image-only pages           |

### Output formats

| `--format`        | Extension | Description                                                     |
|-------------------|:---------:|-----------------------------------------------------------------|
| `text` (default)  | `.txt`    | Plain text extracted from the document                          |
| `md` / `markdown` | `.md`     | Markdown with a `## Page N` heading per page                    |
| `json`            | `.json`   | Raw JSON from liteparse (`ParseResult` serialised)              |

### Supported document types

| Extension                     | How it is parsed                                    | External tool? |
|-------------------------------|-----------------------------------------------------|:--------------:|
| `.pdf`                        | PDFium — spatial text extraction                    | None           |
| `.docx`                       | Pure-Rust ZIP + XML (`word/document.xml`)           | None           |
| `.xlsx` `.xls` `.xlsm` `.ods` | Pure-Rust calamine spreadsheet reader               | None           |
| `.pptx`                       | Pure-Rust ZIP + XML (`ppt/slides/`)                 | None           |
| `.jpg` `.jpeg` `.png` `.webp` | liteparse image + optional Tesseract OCR (`--ocr`) | None           |

> [!NOTE]
> **100% self-contained** — no LibreOffice, no external binaries, no network.
> All formats run natively inside the epax process on both Linux and Windows.

### Examples

**Extract PDF to stdout (plain text):**

```bash
epax parse report.pdf
```

**Save extracted text to a file:**

```bash
epax parse thesis.pdf -o thesis.txt
```

**Save as Markdown — one section per page:**

```bash
epax parse report.pdf --format md -o report.md
epax parse report.pdf -f markdown -o report.md   # same
```

Markdown output for a 3-page document looks like:

```markdown
## Page 1

Introduction text from page 1…

## Page 2

Body text from page 2…

## Page 3

Conclusion text from page 3…
```

**JSON output for downstream processing:**

```bash
epax parse data.pdf --format json -o data.json
```

**Enable OCR for a scanned PDF or image:**

```bash
epax parse scan.pdf --ocr
epax parse scanned.jpg --ocr -o result.txt
```

**Multiple files — concatenated, auto-saved to `parsed.txt`:**

```bash
epax parse a.pdf b.docx c.xlsx
# output: parsed.txt
```

**Multiple files to a single Markdown file:**

```bash
epax parse a.pdf b.pdf c.pdf -f md -o combined.md
```

Each file gets a top-level `# Filename` heading, with pages as `## Page N` subheadings.

---

## `epax inspect`

```
epax inspect <INPUT>
epax info    <INPUT>     # alias
```

Show document metadata without extracting the full text.

### Output

```
file       : report.pdf
pages      : 12
text items : 1 847
characters : 48 302

per-page breakdown:
  page    1     142 items  ████████████████████████████
  page    2     178 items  ███████████████████████████████████
  page    3      93 items  ██████████████████
  page    4     201 items  ████████████████████████████████████████
  ...
```

The bar chart (one `█` per 5 text items, max 40 chars) gives a quick visual
indication of text density per page — useful for spotting scanned pages (few
items) versus text-heavy pages.

### Examples

```bash
epax inspect report.pdf
epax info   report.pdf     # alias
```

---

## Build notes

### Default builds

The `parse` feature is **enabled by default**. A standard build includes it:

```bash
cargo build
cargo build --release
```

### Without parse (pure-Rust binary)

If you don't need document parsing and want to skip PDFium (saves ~10 MB and a
C++ toolchain):

```bash
cargo build --no-default-features
```

This builds the all-Rust core: `compress`, `extract`, `list`, and `squeeze`.

### Platform notes

| Platform        | Requirements                                   |
|-----------------|------------------------------------------------|
| Linux x86-64    | C++ toolchain (gcc/clang) — PDFium is bundled  |
| Windows x86-64  | MSVC toolchain — PDFium ships prebuilt         |
| macOS           | Xcode command-line tools — PDFium is bundled   |

> [!NOTE]
> **MinGW / Windows GNU** targets are not officially supported by liteparse-pdfium.
> Use `--no-default-features` on MinGW, or switch to MSVC.

### OCR (optional)

OCR is disabled by default. When `--ocr` is passed, liteparse calls Tesseract.
To use it:

1. Install Tesseract: `sudo apt install tesseract-ocr` (Linux) or download from
   [tesseract-ocr/tesseract](https://github.com/tesseract-ocr/tesseract/releases) (Windows).
2. Ensure `tesseract` is on your `PATH`.
3. Run: `epax parse scan.pdf --ocr`

For other languages add `--ocr-lang <code>` (not yet exposed in the CLI; set
`TESSDATA_PREFIX` or use the JSON API directly).
