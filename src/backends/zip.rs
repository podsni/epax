use std::fs::File;
use std::io;
use std::path::Path;

use crate::backends::{ListEntry, clamp_level};
use crate::collect::Entry;
use crate::error::{EpaxError, Result};
use crate::util::path::sanitize_entry_path;

/// Create a `.zip` archive containing `entries`.
pub fn compress(output: &Path, entries: &[Entry], level: Option<i32>, verbose: bool) -> Result<()> {
    let file = File::create(output)?;
    let mut writer = zip::ZipWriter::new(file);

    // Deflate is universally supported by readers and pure-Rust here.
    let lvl = clamp_level(level, 0, 9, 6) as i64;
    let options = zip::write::SimpleFileOptions::default()
        .compression_method(zip::CompressionMethod::Deflated)
        .compression_level(Some(lvl));

    for e in entries {
        if verbose {
            println!("  adding: {}", e.arcname);
        }
        writer
            .start_file(&e.arcname, options)
            .map_err(|err| EpaxError::Backend(format!("zip: {err}")))?;
        let mut input = File::open(&e.path)?;
        io::copy(&mut input, &mut writer)?;
    }

    writer
        .finish()
        .map_err(|err| EpaxError::Backend(format!("zip: {err}")))?;
    Ok(())
}

/// Extract a `.zip` archive into `dest`.
pub fn extract(archive: &Path, dest: &Path, verbose: bool) -> Result<()> {
    let file = File::open(archive)?;
    let mut zip = zip::ZipArchive::new(file).map_err(|e| EpaxError::Backend(format!("zip: {e}")))?;

    for i in 0..zip.len() {
        let mut entry = zip
            .by_index(i)
            .map_err(|e| EpaxError::Backend(format!("zip: {e}")))?;
        let out_path = sanitize_entry_path(dest, Path::new(entry.name()))?;

        if entry.is_dir() {
            std::fs::create_dir_all(&out_path)?;
            continue;
        }
        if let Some(parent) = out_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        if verbose {
            println!(" extracting: {}", entry.name());
        }
        let mut out = File::create(&out_path)?;
        io::copy(&mut entry, &mut out)?;

        #[cfg(unix)]
        if let Some(mode) = entry.unix_mode() {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(&out_path, std::fs::Permissions::from_mode(mode))?;
        }
    }
    Ok(())
}

/// List the contents of a `.zip` archive.
pub fn list(archive: &Path) -> Result<Vec<ListEntry>> {
    let file = File::open(archive)?;
    let mut zip = zip::ZipArchive::new(file).map_err(|e| EpaxError::Backend(format!("zip: {e}")))?;
    let mut out = Vec::with_capacity(zip.len());
    for i in 0..zip.len() {
        let entry = zip
            .by_index(i)
            .map_err(|e| EpaxError::Backend(format!("zip: {e}")))?;
        out.push(ListEntry {
            name: entry.name().to_string(),
            size: entry.size(),
        });
    }
    Ok(out)
}
