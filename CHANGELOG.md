# Changelog

All notable changes to the `epax` project will be documented in this file.

## [0.5.2] - 2026-06-04

### Added
- **Multi-Engine Native Rust OCR Support**: Added `--ocr-engine` CLI flag to choose between:
  - `tesseract` (statically linked, default)
  - `ocrs` (pure-Rust OCR engine)
  - `paddle` (PaddleOCR via `ocr-rs` using MNN runtime)
- **Automatic Model Downloads**: Programmed `build.rs` to automatically download ML model files (`.rten`, `.mnn`, and `.txt` keys) if they are missing locally.
- **Embedded Inference Models**: Embedded model bytes directly inside the executable via `include_bytes!`, allowing offline, in-process OCR on both Linux and Windows with 0 external runtimes or library installations.

## [0.5.1] - 2026-06-04

### Added
- **OCR Integration Test**: Added `test_parse_ocr` to `tests/parse.rs` utilizing ImageMagick `convert` to dynamically verify OCR extraction at test time.

### Fixed
- **OCR Feature Enablement**: Explicitly enabled the `tesseract` feature on `liteparse` dependency so that the binary is compiled with Tesseract and Leptonica linked statically.

## [0.5.0] - 2026-06-04

### Added
- **`epax parse` Subcommand**: Extract text from documents locally.
- **`epax inspect` / `epax info` Subcommands**: View structural metadata and page text density (bar chart breakdown) for documents.
- **Pure-Rust Native Office Parsers**:
  - **DOCX**: Native extraction using `quick-xml` (no LibreOffice required).
  - **PPTX**: Native slide sorting and text extraction using `quick-xml` (no LibreOffice required).
  - **XLSX / ODS**: Native spreadsheet reading using `calamine` (no LibreOffice required).
- **Statically Linked Native OCR**: 
  - Enabled by default under the `parse` feature.
  - Links to `libtesseract` and `leptonica` statically in the Rust compilation.
  - Allows 100% self-contained PDF/Image text extraction (`--ocr`) without external applications.
- **Integration Tests**: Added `tests/parse.rs` validating DOCX, PPTX, and ODS parsing.
- **Documentation**: Added [docs/parse.md](docs/parse.md) outlining parse & inspect guides and examples.

### Changed
- **Interactive Prompts**: Improved UX of Guided compression prompts (`epax compress -i`).
- **Dependencies**: Added `quick-xml` and `calamine` for native parsers; enabled `tesseract` feature on `liteparse`.

---

## [0.4.1] - 2026-05-15
- **Fix**: Squeeze size comparison logic and release CI locking failures.

## [0.4.0] - 2026-05-01
- **Feature**: Added interactive compress mode with guided prompts.

## [0.3.0] - 2026-04-10
- **Feature**: Squeeze images, extract-to-folder, and magic-byte detection.

## [0.2.0] - 2026-03-22
- **Feature**: Auto-output compress, magic detection, alias `e` for extract, and install/uninstall scripts.

## [0.1.0] - 2026-03-01
- **Release**: Initial release with core archive formats (zip/7z/tar/gz/bz2/zst).
