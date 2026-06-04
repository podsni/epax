//! `epax parse` and `epax inspect` — document text extraction.
//!
//! # Format routing
//!
//! | Format     | Backend                              | External tool? |
//! |------------|--------------------------------------|----------------|
//! | PDF        | liteparse (PDFium)                   | None           |
//! | DOCX       | zip + quick-xml (`word/document.xml`)| None           |
//! | XLSX       | calamine                             | None           |
//! | PPTX       | zip + quick-xml (`ppt/slides/`)      | None           |
//! | JPG/PNG    | liteparse + optional Tesseract OCR   | None (tesseract optional) |
//!
//! Every format runs entirely locally in-process — no LibreOffice, no network,
//! no external binaries required at runtime on Linux **or** Windows.
//!
//! # Output formats
//!
//! | `--format`  | Extension | Description                                  |
//! |-------------|-----------|----------------------------------------------|
//! | `text`      | `.txt`    | Plain text, pages/sections joined             |
//! | `md`        | `.md`     | Markdown with `## Page N` / `## Sheet N`      |
//! | `json`      | `.json`   | JSON (PDF: liteparse native; others: simple)  |

use std::future::Future;
use std::path::{Path, PathBuf};
use std::pin::Pin;
use std::sync::{Arc, Mutex};

use liteparse::ocr::{OcrEngine, OcrOptions, OcrResult};
use ocrs::{OcrEngine as NativeOcrsEngine, OcrEngineParams, ImageSource, TextItem};
use rten::Model;
use ocr_rs::OcrEngine as NativePaddleEngine;

use crate::error::{EpaxError, Result};

// ── document format detection ─────────────────────────────────────────────────

enum DocFormat {
    Pdf,
    Docx,
    Xlsx,
    Pptx,
    Image, // jpg, jpeg, png, webp — OCR path
}

fn detect_format(path: &Path) -> Option<DocFormat> {
    let ext = path.extension()?.to_string_lossy().to_ascii_lowercase();
    match ext.as_str() {
        "pdf" => Some(DocFormat::Pdf),
        "docx" => Some(DocFormat::Docx),
        "xlsx" | "xls" | "xlsm" | "xlsb" | "ods" => Some(DocFormat::Xlsx),
        "pptx" => Some(DocFormat::Pptx),
        "jpg" | "jpeg" | "png" | "webp" => Some(DocFormat::Image),
        _ => None,
    }
}

// ── output format ─────────────────────────────────────────────────────────────

enum OutFmt {
    Text,
    Markdown,
    Json,
}

impl OutFmt {
    fn from_str(s: &str) -> Result<Self> {
        match s.to_ascii_lowercase().as_str() {
            "text" | "txt" => Ok(Self::Text),
            "md" | "markdown" => Ok(Self::Markdown),
            "json" => Ok(Self::Json),
            other => Err(EpaxError::Backend(format!(
                "unknown parse format '{other}' — expected text, md, or json"
            ))),
        }
    }

    fn ext(&self) -> &'static str {
        match self {
            Self::Text => "txt",
            Self::Markdown => "md",
            Self::Json => "json",
        }
    }
}

// ── DOCX parser (pure Rust) ──────────────────────────────────────────────────

/// Extract text from a DOCX file.
///
/// A DOCX is a ZIP archive containing `word/document.xml`. We extract all
/// `<w:t>` elements, which hold the actual text runs. Paragraph breaks
/// (`<w:p>` elements) are translated to newlines.
fn parse_docx(path: &Path) -> Result<Vec<String>> {
    use quick_xml::events::Event;
    use quick_xml::Reader;
    use std::io::Read;

    let file = std::fs::File::open(path).map_err(EpaxError::Io)?;
    let mut archive = zip::ZipArchive::new(file)
        .map_err(|e| EpaxError::Backend(format!("DOCX zip error: {e}")))?;

    let mut xml_data = String::new();
    archive
        .by_name("word/document.xml")
        .map_err(|_| EpaxError::Backend("DOCX: word/document.xml not found — is this a valid DOCX?".into()))?
        .read_to_string(&mut xml_data)
        .map_err(EpaxError::Io)?;

    let mut reader = Reader::from_str(&xml_data);
    let mut text = String::new();
    let mut in_text_run = false;
    let mut suppress_next_space = false;

    loop {
        match reader.read_event() {
            Ok(Event::Start(ref e) | Event::Empty(ref e)) => {
                match e.local_name().as_ref() {
                    b"t" => in_text_run = true,
                    b"p" => {
                        if !text.is_empty() && !text.ends_with("\n\n") {
                            if text.ends_with('\n') {
                                text.push('\n');
                            } else {
                                text.push_str("\n\n");
                            }
                        }
                        suppress_next_space = true;
                    }
                    b"br" | b"cr" => text.push('\n'),
                    b"tab" => text.push('\t'),
                    _ => {}
                }
            }
            Ok(Event::End(ref e)) => {
                if e.local_name().as_ref() == b"t" {
                    in_text_run = false;
                }
            }
            Ok(Event::Text(e)) => {
                if in_text_run {
                    let chunk = e
                        .decode()
                        .map_err(|e| EpaxError::Backend(format!("DOCX XML decode: {e}")))?;
                    if suppress_next_space {
                        suppress_next_space = false;
                    }
                    text.push_str(&chunk);
                }
            }
            Ok(Event::Eof) => break,
            Err(e) => {
                return Err(EpaxError::Backend(format!("DOCX XML parse error: {e}")));
            }
            _ => {}
        }
    }

    // Trim trailing blank lines
    let trimmed = text.trim_end().to_string();
    Ok(vec![trimmed])
}

// ── XLSX parser (pure Rust via calamine) ─────────────────────────────────────

/// Extract text from an XLSX/XLS/ODS spreadsheet.
///
/// Each worksheet becomes a "section". Cells are joined with tabs per row,
/// rows with newlines. Empty rows are collapsed.
fn parse_xlsx(path: &Path) -> Result<Vec<(String, String)>> {
    use calamine::{open_workbook_auto, Data, Reader};

    let mut workbook = open_workbook_auto(path)
        .map_err(|e| EpaxError::Backend(format!("XLSX open error: {e}")))?;

    let sheet_names: Vec<String> = workbook.sheet_names().to_vec();
    let mut sections: Vec<(String, String)> = Vec::new();

    for name in &sheet_names {
        let range = workbook
            .worksheet_range(name)
            .map_err(|e| EpaxError::Backend(format!("XLSX sheet '{name}': {e}")))?;

        let mut sheet_text = String::new();
        for row in range.rows() {
            let row_str: Vec<String> = row
                .iter()
                .map(|cell| match cell {
                    Data::String(s) => s.clone(),
                    Data::Float(f) => {
                        // Render integers without decimal point
                        if f.fract() == 0.0 && f.abs() < 1e15 {
                            format!("{}", *f as i64)
                        } else {
                            format!("{f}")
                        }
                    }
                    Data::Int(i) => i.to_string(),
                    Data::Bool(b) => (if *b { "TRUE" } else { "FALSE" }).to_string(),
                    Data::DateTime(dt) => dt.to_string(),
                    Data::DateTimeIso(s) | Data::DurationIso(s) => s.clone(),
                    Data::Error(e) => format!("[ERR:{e:?}]"),
                    Data::Empty => String::new(),
                })
                .collect();

            let row_line = row_str.join("\t");
            // Skip entirely empty rows
            if row_str.iter().any(|s| !s.is_empty()) {
                sheet_text.push_str(&row_line);
                sheet_text.push('\n');
            }
        }

        sections.push((name.clone(), sheet_text.trim_end().to_string()));
    }

    Ok(sections)
}

// ── PPTX parser (pure Rust) ──────────────────────────────────────────────────

/// Extract text from a PPTX file.
///
/// A PPTX is a ZIP archive. Each slide is `ppt/slides/slide{N}.xml`.
/// We collect `<a:t>` elements (text runs), grouping by slide.
fn parse_pptx(path: &Path) -> Result<Vec<(usize, String)>> {
    use quick_xml::events::Event;
    use quick_xml::Reader;
    use std::io::Read;

    let file = std::fs::File::open(path).map_err(EpaxError::Io)?;
    let mut archive = zip::ZipArchive::new(file)
        .map_err(|e| EpaxError::Backend(format!("PPTX zip error: {e}")))?;

    // Collect slide file names in order.
    let mut slide_names: Vec<String> = (0..archive.len())
        .filter_map(|i| {
            let entry = archive.by_index(i).ok()?;
            let name = entry.name().to_owned();
            if name.starts_with("ppt/slides/slide") && name.ends_with(".xml") {
                Some(name)
            } else {
                None
            }
        })
        .collect();

    // Sort by slide number (slide1.xml < slide2.xml …)
    slide_names.sort_by_key(|n| {
        n.trim_start_matches("ppt/slides/slide")
            .trim_end_matches(".xml")
            .parse::<u32>()
            .unwrap_or(0)
    });

    let mut slides: Vec<(usize, String)> = Vec::new();

    for (idx, slide_name) in slide_names.iter().enumerate() {
        let mut xml_data = String::new();
        archive
            .by_name(slide_name)
            .map_err(|e| EpaxError::Backend(format!("PPTX: cannot open {slide_name}: {e}")))?
            .read_to_string(&mut xml_data)
            .map_err(EpaxError::Io)?;

        let mut reader = Reader::from_str(&xml_data);
        let mut slide_text = String::new();
        let mut in_text = false;

        loop {
            match reader.read_event() {
                Ok(Event::Start(ref e) | Event::Empty(ref e)) => {
                    match e.local_name().as_ref() {
                        b"t" => in_text = true,
                        b"p" => {
                            if !slide_text.is_empty() && !slide_text.ends_with('\n') {
                                slide_text.push('\n');
                            }
                        }
                        b"br" => slide_text.push('\n'),
                        _ => {}
                    }
                }
                Ok(Event::End(ref e)) => {
                    if e.local_name().as_ref() == b"t" {
                        in_text = false;
                    }
                }
                Ok(Event::Text(e)) => {
                    if in_text {
                        let chunk = e
                            .decode()
                            .map_err(|e| EpaxError::Backend(format!("PPTX XML decode: {e}")))?;
                        slide_text.push_str(&chunk);
                    }
                }
                Ok(Event::Eof) => break,
                Err(e) => {
                    return Err(EpaxError::Backend(format!("PPTX XML parse error at slide {}: {e}", idx + 1)));
                }
                _ => {}
            }
        }

        slides.push((idx + 1, slide_text.trim_end().to_string()));
    }

    Ok(slides)
}

// ── liteparse helpers (PDF + image) ──────────────────────────────────────────

fn make_rt() -> Result<tokio::runtime::Runtime> {
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .map_err(|e| EpaxError::Backend(format!("tokio runtime error: {e}")))
}

fn path_str(p: &Path) -> String {
    p.to_string_lossy().into_owned()
}

// ── rendering ─────────────────────────────────────────────────────────────────

/// A parsed document ready to be serialised.
enum Parsed {
    /// Plain pages (PDF or single-section DOCX).
    Pages(Vec<(String, String)>), // (label, text)
    /// Already-serialised JSON string (from liteparse PDF/JSON mode).
    Json(String),
}

impl Parsed {
    fn render(&self, fmt: &OutFmt) -> String {
        match self {
            Parsed::Json(s) => s.clone(),
            Parsed::Pages(pages) => match fmt {
                OutFmt::Json => {
                    // Simple JSON array of { label, text } objects.
                    let entries: Vec<String> = pages
                        .iter()
                        .map(|(label, text)| {
                            let escaped_label = label.replace('"', "\\\"");
                            let escaped_text = text
                                .replace('\\', "\\\\")
                                .replace('"', "\\\"")
                                .replace('\n', "\\n")
                                .replace('\r', "\\r");
                            format!("  {{\"section\":\"{escaped_label}\",\"text\":\"{escaped_text}\"}}")
                        })
                        .collect();
                    format!("[\n{}\n]\n", entries.join(",\n"))
                }
                OutFmt::Markdown => {
                    let mut md = String::new();
                    for (label, text) in pages {
                        md.push_str(&format!("## {label}\n\n"));
                        let body = text.trim();
                        if body.is_empty() {
                            md.push_str("*(no text)*\n\n");
                        } else {
                            md.push_str(body);
                            md.push_str("\n\n");
                        }
                    }
                    md
                }
                OutFmt::Text => pages
                    .iter()
                    .map(|(_, text)| text.as_str())
                    .collect::<Vec<_>>()
                    .join("\n"),
            },
        }
    }
}

// ── parse a single file ───────────────────────────────────────────────────────

fn parse_file(path: &Path, fmt: &OutFmt, ocr: bool, ocr_engine: &str, ocr_models_dir: Option<&Path>) -> Result<String> {
    let doc_fmt = detect_format(path).ok_or_else(|| {
        EpaxError::Backend(format!(
            "unsupported file type for parse: '{}' — supported: pdf, docx, xlsx, pptx, jpg, png, webp",
            path.display()
        ))
    })?;

    let parsed = match doc_fmt {
        // ── DOCX: pure Rust ─────────────────────────────────────────────────
        DocFormat::Docx => {
            let sections = parse_docx(path)?;
            let pages = sections
                .into_iter()
                .enumerate()
                .map(|(i, text)| (format!("Page {}", i + 1), text))
                .collect();
            Parsed::Pages(pages)
        }

        // ── XLSX: pure Rust via calamine ────────────────────────────────────
        DocFormat::Xlsx => {
            let sheets = parse_xlsx(path)?;
            let pages = sheets
                .into_iter()
                .map(|(name, text)| (format!("Sheet: {name}"), text))
                .collect();
            Parsed::Pages(pages)
        }

        // ── PPTX: pure Rust ─────────────────────────────────────────────────
        DocFormat::Pptx => {
            let slides = parse_pptx(path)?;
            let pages = slides
                .into_iter()
                .map(|(n, text)| (format!("Slide {n}"), text))
                .collect();
            Parsed::Pages(pages)
        }

        // ── PDF / image: liteparse ──────────────────────────────────────────
        DocFormat::Pdf | DocFormat::Image => {
            use liteparse::{LiteParse, LiteParseConfig, OutputFormat};

            let lp_fmt = match fmt {
                OutFmt::Json => OutputFormat::Json,
                _ => OutputFormat::Text,
            };

            let config = LiteParseConfig {
                ocr_enabled: ocr,
                output_format: lp_fmt,
                ..Default::default()
            };
            let mut parser = LiteParse::new(config);

            if ocr {
                match ocr_engine {
                    "ocrs" => {
                        let engine = OcrsEngine::new(ocr_models_dir)
                            .map_err(EpaxError::Backend)?;
                        parser = parser.with_ocr_engine(Arc::new(engine));
                    }
                    "paddle" => {
                        let engine = PaddleOcrEngine::new(ocr_models_dir)
                            .map_err(EpaxError::Backend)?;
                        parser = parser.with_ocr_engine(Arc::new(engine));
                    }
                    "tesseract" | _ => {
                        #[cfg(windows)]
                        {
                            return Err(EpaxError::Backend(
                                "Tesseract OCR engine is not supported on Windows. Please use '--ocr-engine ocrs' or '--ocr-engine paddle' instead.".to_string()
                            ));
                        }
                        #[cfg(not(windows))]
                        {} // default to tesseract
                    }
                }
            }

            let rt = make_rt()?;

            let result = rt
                .block_on(parser.parse(&path_str(path)))
                .map_err(|e| EpaxError::Backend(format!("parse '{}': {e}", path.display())))?;

            match fmt {
                OutFmt::Json => Parsed::Json(result.text),
                _ => {
                    let pages = result
                        .pages
                        .iter()
                        .map(|p| (format!("Page {}", p.page_number), p.text.clone()))
                        .collect();
                    Parsed::Pages(pages)
                }
            }
        }
    };

    Ok(parsed.render(fmt))
}

// ── public API ────────────────────────────────────────────────────────────────

/// Run `epax parse`: extract text from one or more documents.
pub fn run(
    inputs: &[PathBuf],
    output: Option<&Path>,
    format: &str,
    ocr: bool,
    ocr_engine: &str,
    ocr_models_dir: Option<&Path>,
) -> Result<()> {
    let out_fmt = OutFmt::from_str(format)?;
    let multiple = inputs.len() > 1;
    let mut buf = String::new();

    for path in inputs {
        if !path.exists() {
            return Err(EpaxError::MissingInput(path.clone()));
        }

        let text = parse_file(path, &out_fmt, ocr, ocr_engine, ocr_models_dir)?;

        if !buf.is_empty() {
            buf.push('\n');
        }

        // For Markdown multi-file, add a top-level file heading.
        if multiple && matches!(out_fmt, OutFmt::Markdown) {
            let label = path
                .file_name()
                .map(|n| n.to_string_lossy().into_owned())
                .unwrap_or_else(|| path.display().to_string());
            buf.push_str(&format!("# {label}\n\n"));
        }

        buf.push_str(&text);
        if !buf.ends_with('\n') {
            buf.push('\n');
        }
    }

    match output {
        Some(dest) => write_output(dest, &buf)?,
        None if multiple => {
            let auto = PathBuf::from(format!("parsed.{}", out_fmt.ext()));
            write_output(&auto, &buf)?;
        }
        None => {
            use std::io::Write as _;
            std::io::stdout()
                .write_all(buf.as_bytes())
                .map_err(EpaxError::Io)?;
        }
    }

    Ok(())
}

/// Write `content` to `dest`, creating parent directories as needed.
fn write_output(dest: &Path, content: &str) -> Result<()> {
    if let Some(parent) = dest.parent()
        && !parent.as_os_str().is_empty()
    {
        std::fs::create_dir_all(parent).map_err(EpaxError::Io)?;
    }
    std::fs::write(dest, content.as_bytes()).map_err(EpaxError::Io)?;
    eprintln!("wrote {}", dest.display());
    Ok(())
}

/// Run `epax inspect`: print structural metadata for a document.
pub fn inspect(input: &Path) -> Result<()> {
    if !input.exists() {
        return Err(EpaxError::MissingInput(input.to_path_buf()));
    }

    let doc_fmt = detect_format(input).ok_or_else(|| {
        EpaxError::Backend(format!(
            "unsupported file type: '{}' — supported: pdf, docx, xlsx, pptx, jpg, png, webp",
            input.display()
        ))
    })?;

    match doc_fmt {
        // ── DOCX inspect ────────────────────────────────────────────────────
        DocFormat::Docx => {
            let sections = parse_docx(input)?;
            println!("file    : {}", input.display());
            println!("format  : DOCX (native Rust parser)");
            println!("sections: {}", sections.len());
            for (i, text) in sections.iter().enumerate() {
                let words = text.split_whitespace().count();
                let chars = text.len();
                println!("  section {}: {} words, {} chars", i + 1, words, chars);
            }
        }

        // ── XLSX inspect ────────────────────────────────────────────────────
        DocFormat::Xlsx => {
            let sheets = parse_xlsx(input)?;
            println!("file    : {}", input.display());
            println!("format  : XLSX (native Rust parser)");
            println!("sheets  : {}", sheets.len());
            for (name, text) in &sheets {
                let rows = text.lines().count();
                let words = text.split_whitespace().count();
                println!("  sheet '{name}': {rows} rows, {words} words");
            }
        }

        // ── PPTX inspect ────────────────────────────────────────────────────
        DocFormat::Pptx => {
            let slides = parse_pptx(input)?;
            println!("file    : {}", input.display());
            println!("format  : PPTX (native Rust parser)");
            println!("slides  : {}", slides.len());
            for (n, text) in &slides {
                let words = text.split_whitespace().count();
                println!("  slide {n}: {words} words");
            }
        }

        // ── PDF / image inspect (liteparse) ─────────────────────────────────
        DocFormat::Pdf | DocFormat::Image => {
            use liteparse::{LiteParse, LiteParseConfig};

            let config = LiteParseConfig {
                ocr_enabled: false,
                ..Default::default()
            };
            let parser = LiteParse::new(config);
            let rt = make_rt()?;

            let result = rt
                .block_on(parser.parse(&path_str(input)))
                .map_err(|e| EpaxError::Backend(format!("inspect '{}': {e}", input.display())))?;

            let total_items: usize = result.pages.iter().map(|p| p.text_items.len()).sum();
            let total_chars: usize = result.text.len();

            println!("file       : {}", input.display());
            println!("format     : PDF (PDFium)");
            println!("pages      : {}", result.pages.len());
            println!("text items : {total_items}");
            println!("characters : {total_chars}");

            if !result.pages.is_empty() {
                println!();
                println!("per-page breakdown:");
                let n_pages = result.pages.len();
                let page_col = n_pages.to_string().len().max(4);
                for page in &result.pages {
                    let items = page.text_items.len();
                    let bar_len = (items / 5).min(40);
                    let bar: String = "█".repeat(bar_len);
                    let page_num = page.page_number;
                    let plural = if items == 1 { " " } else { "s" };
                    println!("  page {page_num:>page_col$}  {items:>5} item{plural}  {bar}");
                }
            }
        }
    }

    Ok(())
}

// ── OcrsEngine (native Rust) ──────────────────────────────────────────────────

static DET_MODEL_BYTES: &[u8] = include_bytes!("../../models/text-detection.rten");
static REC_MODEL_BYTES: &[u8] = include_bytes!("../../models/text-recognition.rten");

fn dirs_data() -> Option<std::path::PathBuf> {
    #[cfg(unix)]
    {
        std::env::var_os("HOME").map(|h| std::path::PathBuf::from(h).join(".local/share/epax"))
    }
    #[cfg(windows)]
    {
        std::env::var_os("LOCALAPPDATA").map(|a| std::path::PathBuf::from(a).join("epax"))
    }
    #[cfg(not(any(unix, windows)))]
    {
        None
    }
}

fn resolve_models_dir(custom_dir: Option<&Path>) -> PathBuf {
    if let Some(dir) = custom_dir {
        dir.to_path_buf()
    } else if let Ok(val) = std::env::var("EPAX_OCR_MODELS_DIR") {
        PathBuf::from(val)
    } else {
        let default_base = dirs_data().unwrap_or_else(|| PathBuf::from("."));
        default_base.join("models")
    }
}

fn ensure_model_file(dir: &Path, filename: &str, url: &str) -> std::result::Result<PathBuf, String> {
    let dest_path = dir.join(filename);
    if dest_path.exists() {
        return Ok(dest_path);
    }

    std::fs::create_dir_all(dir)
        .map_err(|e| format!("Failed to create models directory {:?}: {}", dir, e))?;

    eprintln!("OCR model file {:?} not found. Downloading from {}...", filename, url);

    let status = std::process::Command::new("curl")
        .args(["-L", "-o", dest_path.to_str().unwrap(), url])
        .status()
        .map_err(|e| format!("failed to execute curl: {}", e))?;

    if !status.success() {
        return Err(format!("failed to download model from {}", url));
    }

    Ok(dest_path)
}

struct OcrsEngine {
    engine: NativeOcrsEngine,
}

impl OcrsEngine {
    fn new(custom_dir: Option<&Path>) -> std::result::Result<Self, String> {
        let models_dir = resolve_models_dir(custom_dir);

        let det_path = ensure_model_file(
            &models_dir,
            "text-detection.rten",
            "https://ocrs-models.s3-accelerate.amazonaws.com/text-detection.rten",
        );

        let det_model = match det_path {
            Ok(path) => Model::load_file(path)
                .map_err(|e| format!("Failed to load text detection model from file: {:?}", e))?,
            Err(e) => {
                eprintln!("Warning: could not retrieve detection model from disk/network ({e}). Falling back to embedded model.");
                Model::load_static_slice(DET_MODEL_BYTES)
                    .map_err(|err| format!("Failed to load embedded text detection model: {:?}", err))?
            }
        };

        let rec_path = ensure_model_file(
            &models_dir,
            "text-recognition.rten",
            "https://ocrs-models.s3-accelerate.amazonaws.com/text-recognition.rten",
        );

        let rec_model = match rec_path {
            Ok(path) => Model::load_file(path)
                .map_err(|e| format!("Failed to load text recognition model from file: {:?}", e))?,
            Err(e) => {
                eprintln!("Warning: could not retrieve recognition model from disk/network ({e}). Falling back to embedded model.");
                Model::load_static_slice(REC_MODEL_BYTES)
                    .map_err(|err| format!("Failed to load embedded text recognition model: {:?}", err))?
            }
        };

        let engine = NativeOcrsEngine::new(OcrEngineParams {
            detection_model: Some(det_model),
            recognition_model: Some(rec_model),
            ..Default::default()
        })
        .map_err(|e| format!("Failed to create ocrs engine: {:?}", e))?;

        Ok(Self { engine })
    }
}

impl OcrEngine for OcrsEngine {
    fn name(&self) -> &str {
        "ocrs"
    }

    fn recognize<'a, 'b: 'a, 'c: 'a>(
        &'a self,
        image_data: &'c [u8],
        width: u32,
        height: u32,
        _options: &'b OcrOptions,
    ) -> Pin<
        Box<
            dyn Future<Output = std::result::Result<Vec<OcrResult>, Box<dyn std::error::Error + Send + Sync>>>
                + Send
                + '_,
        >,
    > {
        Box::pin(async move {
            let img_source = ImageSource::from_bytes(image_data, (width, height))
                .map_err(|e| Box::new(std::io::Error::new(std::io::ErrorKind::InvalidInput, format!("ImageSource error: {:?}", e))) as Box<dyn std::error::Error + Send + Sync>)?;

            let ocr_input = self.engine.prepare_input(img_source)
                .map_err(|e| Box::new(std::io::Error::other(format!("prepare_input error: {:?}", e))) as Box<dyn std::error::Error + Send + Sync>)?;

            let word_rects = self.engine.detect_words(&ocr_input)
                .map_err(|e| Box::new(std::io::Error::other(format!("detect_words error: {:?}", e))) as Box<dyn std::error::Error + Send + Sync>)?;

            let line_rects = self.engine.find_text_lines(&ocr_input, &word_rects);

            let lines = self.engine.recognize_text(&ocr_input, &line_rects)
                .map_err(|e| Box::new(std::io::Error::other(format!("recognize_text error: {:?}", e))) as Box<dyn std::error::Error + Send + Sync>)?;

            let mut results = Vec::new();
            for line in lines.into_iter().flatten() {
                for word in line.words() {
                    let rect = word.bounding_rect();
                    let left = rect.left() as f32;
                    let top = rect.top() as f32;
                    let right = rect.right() as f32;
                    let bottom = rect.bottom() as f32;

                    results.push(OcrResult {
                        text: word.to_string(),
                        bbox: [left, top, right, bottom],
                        confidence: 1.0,
                    });
                }
            }

            Ok(results)
        })
    }
}

// ── PaddleOcrEngine (native Rust / MNN) ───────────────────────────────────────

static PAD_DET_BYTES: &[u8] = include_bytes!("../../models/PP-OCRv5_mobile_det_fp16.mnn");
static PAD_REC_BYTES: &[u8] = include_bytes!("../../models/PP-OCRv5_mobile_rec_fp16.mnn");
static PAD_KEYS_BYTES: &[u8] = include_bytes!("../../models/ppocr_keys_v5.txt");

struct PaddleOcrEngine {
    engine: Mutex<NativePaddleEngine>,
}

impl PaddleOcrEngine {
    fn new(custom_dir: Option<&Path>) -> std::result::Result<Self, String> {
        let models_dir = resolve_models_dir(custom_dir);

        let det_res = ensure_model_file(
            &models_dir,
            "PP-OCRv5_mobile_det_fp16.mnn",
            "https://raw.githubusercontent.com/zibo-chen/rust-paddle-ocr/next/models/PP-OCRv5_mobile_det_fp16.mnn",
        );
        let rec_res = ensure_model_file(
            &models_dir,
            "PP-OCRv5_mobile_rec_fp16.mnn",
            "https://raw.githubusercontent.com/zibo-chen/rust-paddle-ocr/next/models/PP-OCRv5_mobile_rec_fp16.mnn",
        );
        let keys_res = ensure_model_file(
            &models_dir,
            "ppocr_keys_v5.txt",
            "https://raw.githubusercontent.com/zibo-chen/rust-paddle-ocr/next/models/ppocr_keys_v5.txt",
        );

        let engine = match (det_res, rec_res, keys_res) {
            (Ok(det_path), Ok(rec_path), Ok(keys_path)) => {
                NativePaddleEngine::new(det_path, rec_path, keys_path, None)
                    .map_err(|e| format!("Failed to create PaddleOCR engine from files: {:?}", e))?
            }
            _ => {
                eprintln!("Warning: could not retrieve PaddleOCR models from disk/network. Falling back to embedded models.");
                NativePaddleEngine::from_bytes(
                    PAD_DET_BYTES,
                    PAD_REC_BYTES,
                    PAD_KEYS_BYTES,
                    None,
                )
                .map_err(|e| format!("Failed to create PaddleOCR engine from embedded bytes: {:?}", e))?
            }
        };

        Ok(Self {
            engine: Mutex::new(engine),
        })
    }
}

impl OcrEngine for PaddleOcrEngine {
    fn name(&self) -> &str {
        "paddle"
    }

    fn recognize<'a, 'b: 'a, 'c: 'a>(
        &'a self,
        image_data: &'c [u8],
        width: u32,
        height: u32,
        _options: &'b OcrOptions,
    ) -> Pin<
        Box<
            dyn Future<Output = std::result::Result<Vec<OcrResult>, Box<dyn std::error::Error + Send + Sync>>>
                + Send
                + '_,
        >,
    > {
        Box::pin(async move {
            let engine = self.engine.lock().map_err(|e| {
                Box::new(std::io::Error::other(
                    format!("Mutex lock failed: {}", e),
                )) as Box<dyn std::error::Error + Send + Sync>
            })?;

            let rgb_img = image::ImageBuffer::<image::Rgb<u8>, _>::from_raw(width, height, image_data.to_vec())
                .ok_or_else(|| {
                    Box::new(std::io::Error::new(
                        std::io::ErrorKind::InvalidInput,
                        "Failed to create ImageBuffer from raw RGB bytes",
                    )) as Box<dyn std::error::Error + Send + Sync>
                })?;
            let dyn_img = image::DynamicImage::ImageRgb8(rgb_img);

            let native_results = engine.recognize(&dyn_img)
                .map_err(|e| Box::new(std::io::Error::other(format!("PaddleOCR recognize error: {:?}", e))) as Box<dyn std::error::Error + Send + Sync>)?;

            let results: Vec<OcrResult> = native_results
                .into_iter()
                .map(|r| {
                    let left = r.bbox.rect.left() as f32;
                    let top = r.bbox.rect.top() as f32;
                    let right = left + r.bbox.rect.width() as f32;
                    let bottom = top + r.bbox.rect.height() as f32;
                    OcrResult {
                        text: r.text,
                        bbox: [left, top, right, bottom],
                        confidence: r.confidence,
                    }
                })
                .collect();

            Ok(results)
        })
    }
}
