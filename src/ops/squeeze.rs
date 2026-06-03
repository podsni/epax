use std::path::{Path, PathBuf};

use crate::error::{EpaxError, Result};

/// Supported image extensions.
const SUPPORTED_EXTS: &[&str] = &["jpg", "jpeg", "png", "webp"];

/// Parse the user-provided format string into an image target format.
fn target_format(name: &str) -> Result<image::ImageFormat> {
    match name.to_ascii_lowercase().as_str() {
        "webp" => Ok(image::ImageFormat::WebP),
        "jpeg" | "jpg" => Ok(image::ImageFormat::Jpeg),
        "png" => Ok(image::ImageFormat::Png),
        other => Err(EpaxError::Backend(format!(
            "unknown image format '{other}' (expected webp, jpeg, or png)"
        ))),
    }
}

/// Return the file extension for a target format.
fn format_ext(fmt: image::ImageFormat) -> &'static str {
    match fmt {
        image::ImageFormat::WebP => "webp",
        image::ImageFormat::Jpeg => "jpg",
        image::ImageFormat::Png => "png",
        _ => "bin",
    }
}

fn is_supported_image(path: &Path) -> bool {
    path.extension()
        .map(|e| {
            let e = e.to_string_lossy().to_ascii_lowercase();
            SUPPORTED_EXTS.contains(&e.as_str())
        })
        .unwrap_or(false)
}

/// Replace the extension on a file name with one matching the target format.
fn with_format_ext(name: &str, fmt: image::ImageFormat) -> String {
    let stem = name
        .rfind('.')
        .map(|p| &name[..p])
        .unwrap_or(name);
    format!("{}.{}", stem, format_ext(fmt))
}

/// Format a file size in human-readable form.
fn human_size(bytes: u64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB"];
    let mut size = bytes as f64;
    let mut unit = 0;
    while size > 1024.0 && unit < UNITS.len() - 1 {
        size /= 1024.0;
        unit += 1;
    }
    format!("{:.1} {}", size, UNITS[unit])
}

/// Re-encode a single image file. Returns the compressed file size in bytes.
fn process_image(src: &Path, dst: &Path, fmt: image::ImageFormat, quality: u8) -> Result<u64> {
    let img = image::open(src)
        .map_err(|e: image::ImageError| EpaxError::Backend(e.to_string()))?;

    match fmt {
        image::ImageFormat::WebP => {
            img.save_with_format(dst, image::ImageFormat::WebP)
                .map_err(|e| EpaxError::Backend(e.to_string()))?;
        }
        image::ImageFormat::Jpeg => {
            // Use low-level encoder for quality control.
            use image::codecs::jpeg::JpegEncoder;
            let rgba = img.to_rgba8();
            let (w, h) = rgba.dimensions();
            let mut output = std::fs::File::create(dst).map_err(EpaxError::Io)?;
            JpegEncoder::new_with_quality(&mut output, quality)
                .encode(rgba.as_raw(), w, h, image::ExtendedColorType::Rgba8)
                .map_err(|e: image::ImageError| EpaxError::Backend(e.to_string()))?;
        }
        image::ImageFormat::Png => {
            let mut output = std::fs::File::create(dst).map_err(EpaxError::Io)?;
            img.write_to(&mut output, image::ImageFormat::Png)
                .map_err(|e| EpaxError::Backend(e.to_string()))?;
        }
        _ => {
            img.save(dst)
                .map_err(|e| EpaxError::Backend(e.to_string()))?;
        }
    }

    let compressed_size = std::fs::metadata(dst)
        .map_err(EpaxError::Io)?
        .len();
    Ok(compressed_size)
}

/// Run the squeeze command.
pub fn run(inputs: &[PathBuf], output: &Path, format: &str, quality: u8) -> Result<()> {
    let target_fmt = target_format(format)?;

    std::fs::create_dir_all(output).map_err(EpaxError::Io)?;

    struct Result {
        name: String,
        original: u64,
        compressed: u64,
    }
    let mut results: Vec<Result> = Vec::new();

    for input in inputs {
        if input.is_dir() {
            for entry in walkdir::WalkDir::new(input)
                .into_iter()
                .filter_map(|e| e.ok())
            {
                if entry.file_type().is_file() && is_supported_image(entry.path()) {
                    let original_size = std::fs::metadata(entry.path())
                        .map_err(EpaxError::Io)?
                        .len();
                    let relative = entry
                        .path()
                        .strip_prefix(input)
                        .unwrap_or(entry.path())
                        .to_path_buf();
                    let parent = relative.parent().unwrap_or(Path::new(""));
                    let dest_dir = output.join(parent);
                    std::fs::create_dir_all(&dest_dir).map_err(EpaxError::Io)?;

                    let old_name = relative
                        .file_name()
                        .map(|s| s.to_string_lossy().into_owned())
                        .unwrap_or_default();
                    let new_name = with_format_ext(&old_name, target_fmt);
                    let dest = dest_dir.join(&new_name);

                    let compressed_size = process_image(entry.path(), &dest, target_fmt, quality)?;
                    results.push(Result { name: old_name, original: original_size, compressed: compressed_size });
                }
            }
        } else if is_supported_image(input) {
            let original_size = std::fs::metadata(input)
                .map_err(EpaxError::Io)?
                .len();
            let old_name = input
                .file_name()
                .map(|s| s.to_string_lossy().into_owned())
                .unwrap_or_default();
            let new_name = with_format_ext(&old_name, target_fmt);
            let dest = output.join(&new_name);

            let compressed_size = process_image(input, &dest, target_fmt, quality)?;
            results.push(Result { name: old_name, original: original_size, compressed: compressed_size });
        } else {
            eprintln!("warning: skipping unsupported file: {}", input.display());
        }
    }

    if results.is_empty() {
        return Err(EpaxError::Backend(
            "no supported image files found (try .jpg, .png, .webp)".to_string(),
        ));
    }

    // Summary with size comparison
    println!();
    for r in &results {
        let saved = r.original.saturating_sub(r.compressed);
        let pct = if r.original > 0 {
            (saved as f64 / r.original as f64 * 100.0) as i32
        } else {
            0
        };
        let (arrow, note) = if saved > 0 { ("↓", format!("{}% smaller", pct)) } else { ("=", "no change".to_string()) };
        println!(
            "  {}  {}  ({} → {}, {})",
            arrow,
            r.name,
            human_size(r.original),
            human_size(r.compressed),
            note,
        );
    }

    let total_orig: u64 = results.iter().map(|r| r.original).sum();
    let total_comp: u64 = results.iter().map(|r| r.compressed).sum();
    let total_saved = total_orig.saturating_sub(total_comp);
    let total_pct = if total_orig > 0 {
        (total_saved as f64 / total_orig as f64 * 100.0) as i32
    } else {
        0
    };

    println!(
        "  ──\n  {} image{}: {} → {}  ({}% smaller)",
        results.len(),
        if results.len() == 1 { "" } else { "s" },
        human_size(total_orig),
        human_size(total_comp),
        total_pct,
    );
    println!("  output: {}/*.{}", output.display(), format_ext(target_fmt));

    Ok(())
}
