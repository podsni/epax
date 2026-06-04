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

### OCR (Optional but Built-in)

OCR is disabled by default. When `--ocr` is passed, you can choose between three different OCR engines:
1. **`tesseract`** (Default): Classic, high-accuracy external OCR engine. Requires `tesseract` binary to be installed on your system.
2. **`ocrs`**: Pure-Rust neural-network-based OCR engine running natively in-process. Requires no external dependencies.
3. **`paddle`**: High-performance layout-aware OCR engine running natively in-process via MNN runtime (via `ocr-rs` wrapper).

To run OCR:
```bash
# Use default Tesseract (requires tesseract system binary)
epax parse scan.pdf --ocr

# Use pure-Rust in-process engine
epax parse scan.pdf --ocr --ocr-engine ocrs

# Use PaddleOCR in-process engine
epax parse scan.pdf --ocr --ocr-engine paddle
```

---

### OCR Model Locations & Custom Downloads

When using the native engines (`ocrs` or `paddle`), `epax` dynamically resolves where to load/save model weights.

#### Directory Resolution Order
1. **Command Line Flag**: `--ocr-models-dir <DIR>`
2. **Environment Variable**: `EPAX_OCR_MODELS_DIR`
3. **Default System Directory**:
   - **Linux / macOS**: `~/.local/share/epax/models/`
   - **Windows**: `%LOCALAPPDATA%\epax\models\` (resolves to `C:\Users\<Username>\AppData\Local\epax\models\`)

If models are missing from the resolved directory, `epax` will automatically attempt to download them. If there is no internet connection, it automatically falls back to utilizing the model weights statically embedded directly inside the compiled binary.

---

### Manual Downloads & Air-Gapped Setup

In air-gapped environments or restricted networks, download the weights manually and place them in the models folder.

#### 1. `ocrs` Engine Models
* **Text Detection Model** (`text-detection.rten`):
  * **Link**: [text-detection.rten](https://ocrs-models.s3-accelerate.amazonaws.com/text-detection.rten)
  * **Download**: `curl -L -o text-detection.rten https://ocrs-models.s3-accelerate.amazonaws.com/text-detection.rten`
* **Text Recognition Model** (`text-recognition.rten`):
  * **Link**: [text-recognition.rten](https://ocrs-models.s3-accelerate.amazonaws.com/text-recognition.rten)
  * **Download**: `curl -L -o text-recognition.rten https://ocrs-models.s3-accelerate.amazonaws.com/text-recognition.rten`

#### 2. `paddle` Engine Models
* **Detection Model** (`PP-OCRv5_mobile_det_fp16.mnn`):
  * **Link**: [PP-OCRv5_mobile_det_fp16.mnn](https://raw.githubusercontent.com/zibo-chen/rust-paddle-ocr/next/models/PP-OCRv5_mobile_det_fp16.mnn)
  * **Download**: `curl -L -o PP-OCRv5_mobile_det_fp16.mnn https://raw.githubusercontent.com/zibo-chen/rust-paddle-ocr/next/models/PP-OCRv5_mobile_det_fp16.mnn`
* **Recognition Model** (`PP-OCRv5_mobile_rec_fp16.mnn`):
  * **Link**: [PP-OCRv5_mobile_rec_fp16.mnn](https://raw.githubusercontent.com/zibo-chen/rust-paddle-ocr/next/models/PP-OCRv5_mobile_rec_fp16.mnn)
  * **Download**: `curl -L -o PP-OCRv5_mobile_rec_fp16.mnn https://raw.githubusercontent.com/zibo-chen/rust-paddle-ocr/next/models/PP-OCRv5_mobile_rec_fp16.mnn`
* **Character Keys Map** (`ppocr_keys_v5.txt`):
  * **Link**: [ppocr_keys_v5.txt](https://raw.githubusercontent.com/zibo-chen/rust-paddle-ocr/next/models/ppocr_keys_v5.txt)
  * **Download**: `curl -L -o ppocr_keys_v5.txt https://raw.githubusercontent.com/zibo-chen/rust-paddle-ocr/next/models/ppocr_keys_v5.txt`

#### Quick Setup Example (Linux/macOS)
```bash
mkdir -p ~/.local/share/epax/models
cd ~/.local/share/epax/models
curl -L -O https://ocrs-models.s3-accelerate.amazonaws.com/text-detection.rten
curl -L -O https://ocrs-models.s3-accelerate.amazonaws.com/text-recognition.rten
curl -L -O https://raw.githubusercontent.com/zibo-chen/rust-paddle-ocr/next/models/PP-OCRv5_mobile_det_fp16.mnn
curl -L -O https://raw.githubusercontent.com/zibo-chen/rust-paddle-ocr/next/models/PP-OCRv5_mobile_rec_fp16.mnn
curl -L -O https://raw.githubusercontent.com/zibo-chen/rust-paddle-ocr/next/models/ppocr_keys_v5.txt
```

#### Quick Setup Example (Windows PowerShell)
```powershell
New-Item -ItemType Directory -Force -Path "$env:LOCALAPPDATA\epax\models"
Set-Location -Path "$env:LOCALAPPDATA\epax\models"
Invoke-WebRequest -Uri "https://ocrs-models.s3-accelerate.amazonaws.com/text-detection.rten" -OutFile "text-detection.rten"
Invoke-WebRequest -Uri "https://ocrs-models.s3-accelerate.amazonaws.com/text-recognition.rten" -OutFile "text-recognition.rten"
Invoke-WebRequest -Uri "https://raw.githubusercontent.com/zibo-chen/rust-paddle-ocr/next/models/PP-OCRv5_mobile_det_fp16.mnn" -OutFile "PP-OCRv5_mobile_det_fp16.mnn"
Invoke-WebRequest -Uri "https://raw.githubusercontent.com/zibo-chen/rust-paddle-ocr/next/models/PP-OCRv5_mobile_rec_fp16.mnn" -OutFile "PP-OCRv5_mobile_rec_fp16.mnn"
Invoke-WebRequest -Uri "https://raw.githubusercontent.com/zibo-chen/rust-paddle-ocr/next/models/ppocr_keys_v5.txt" -OutFile "ppocr_keys_v5.txt"
```

