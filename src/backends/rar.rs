use std::path::Path;

use unrar::Archive;

use crate::backends::ListEntry;
use crate::error::{EpaxError, Result};
use crate::util::path::sanitize_entry_path;

/// Extract a `.rar` archive into `dest`. RAR creation is impossible (proprietary
/// format), so only extraction and listing are provided.
pub fn extract(archive: &Path, dest: &Path, verbose: bool) -> Result<()> {
    std::fs::create_dir_all(dest)?;

    let mut open = Archive::new(&archive)
        .open_for_processing()
        .map_err(|e| EpaxError::Backend(format!("rar: {e}")))?;

    while let Some(header) = open
        .read_header()
        .map_err(|e| EpaxError::Backend(format!("rar: {e}")))?
    {
        let entry = header.entry();
        // Guard against path traversal before letting unrar write to disk.
        sanitize_entry_path(dest, &entry.filename)?;

        if verbose && entry.is_file() {
            println!(" extracting: {}", entry.filename.display());
        }

        open = header
            .extract_with_base(dest)
            .map_err(|e| EpaxError::Backend(format!("rar: {e}")))?;
    }
    Ok(())
}

/// List a `.rar` archive's entries.
pub fn list(archive: &Path) -> Result<Vec<ListEntry>> {
    let mut out = Vec::new();
    let listing = Archive::new(&archive)
        .open_for_listing()
        .map_err(|e| EpaxError::Backend(format!("rar: {e}")))?;
    for header in listing {
        let header = header.map_err(|e| EpaxError::Backend(format!("rar: {e}")))?;
        out.push(ListEntry {
            name: header.filename.display().to_string(),
            size: header.unpacked_size,
        });
    }
    Ok(out)
}
