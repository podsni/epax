use std::fs::File;
use std::path::Path;

use sevenz_rust2::{ArchiveEntry, ArchiveWriter};

use crate::backends::ListEntry;
use crate::collect::Entry;
use crate::error::{EpaxError, Result};
use crate::util::path::sanitize_entry_path;

/// Create a `.7z` archive (LZMA2) containing `entries`.
pub fn compress(output: &Path, entries: &[Entry], _level: Option<i32>, verbose: bool) -> Result<()> {
    let mut writer =
        ArchiveWriter::create(output).map_err(|e| EpaxError::Backend(format!("7z: {e}")))?;

    for e in entries {
        if verbose {
            println!("  adding: {}", e.arcname);
        }
        let reader = File::open(&e.path)?;
        let entry = ArchiveEntry::from_path(&e.path, e.arcname.clone());
        writer
            .push_archive_entry(entry, Some(reader))
            .map_err(|err| EpaxError::Backend(format!("7z: {err}")))?;
    }

    writer
        .finish()
        .map_err(|e| EpaxError::Backend(format!("7z: {e}")))?;
    Ok(())
}

/// Extract a `.7z` archive into `dest`, sanitizing each entry path.
pub fn extract(archive: &Path, dest: &Path, verbose: bool) -> Result<()> {
    use std::io;

    let dest_root = dest.to_path_buf();
    std::fs::create_dir_all(&dest_root)?;

    sevenz_rust2::decompress_file_with_extract_fn(archive, dest, move |entry, reader, _default| {
        // Recompute a safe path from the entry name; ignore the library's join.
        let safe = sanitize_entry_path(&dest_root, Path::new(entry.name()))
            .map_err(|e| sevenz_rust2::Error::Other(e.to_string().into()))?;

        if entry.is_directory() {
            std::fs::create_dir_all(&safe)?;
        } else {
            if let Some(parent) = safe.parent() {
                std::fs::create_dir_all(parent)?;
            }
            if verbose {
                println!(" extracting: {}", entry.name());
            }
            let mut out = File::create(&safe)?;
            io::copy(reader, &mut out)?;
        }
        Ok(true)
    })
    .map_err(|e| EpaxError::Backend(format!("7z: {e}")))?;
    Ok(())
}

/// List a `.7z` archive's entries.
pub fn list(archive: &Path) -> Result<Vec<ListEntry>> {
    let arc =
        sevenz_rust2::Archive::open(archive).map_err(|e| EpaxError::Backend(format!("7z: {e}")))?;
    Ok(arc
        .files
        .iter()
        .map(|f| ListEntry {
            name: f.name().to_string(),
            size: f.size(),
        })
        .collect())
}
