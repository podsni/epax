use std::fs::File;
use std::io::{Read, Write};
use std::path::Path;

use crate::backends::ListEntry;
use crate::collect::Entry;
use crate::error::Result;
use crate::util::path::sanitize_entry_path;

/// Write `entries` as a tar stream into `w`, returning the finished writer.
/// Shared by the plain-tar backend and the gz/bz2/zst stream backend.
pub fn write_entries<W: Write>(w: W, entries: &[Entry], verbose: bool) -> Result<W> {
    let mut builder = tar::Builder::new(w);
    for e in entries {
        if verbose {
            println!("  adding: {}", e.arcname);
        }
        builder.append_path_with_name(&e.path, &e.arcname)?;
    }
    // into_inner finishes the archive (writes the trailer) and returns the writer.
    Ok(builder.into_inner()?)
}

/// Extract a tar stream from `r` into `dest`, guarding every entry path.
pub fn extract_reader<R: Read>(r: R, dest: &Path, verbose: bool) -> Result<()> {
    let mut archive = tar::Archive::new(r);
    for entry in archive.entries()? {
        let mut entry = entry?;
        let path = entry.path()?.into_owned();
        let safe = sanitize_entry_path(dest, &path)?;
        if let Some(parent) = safe.parent() {
            std::fs::create_dir_all(parent)?;
        }
        if verbose {
            println!(" extracting: {}", path.display());
        }
        entry.unpack(&safe)?;
    }
    Ok(())
}

/// List a tar stream's entries.
pub fn list_reader<R: Read>(r: R) -> Result<Vec<ListEntry>> {
    let mut archive = tar::Archive::new(r);
    let mut out = Vec::new();
    for entry in archive.entries()? {
        let entry = entry?;
        out.push(ListEntry {
            name: entry.path()?.display().to_string(),
            size: entry.size(),
        });
    }
    Ok(out)
}

/// Create a plain `.tar` archive (no compression).
pub fn compress(output: &Path, entries: &[Entry], verbose: bool) -> Result<()> {
    let file = File::create(output)?;
    let _ = write_entries(file, entries, verbose)?;
    Ok(())
}

/// Extract a plain `.tar` archive.
pub fn extract(archive: &Path, dest: &Path, verbose: bool) -> Result<()> {
    std::fs::create_dir_all(dest)?;
    let file = File::open(archive)?;
    extract_reader(file, dest, verbose)
}

/// List a plain `.tar` archive.
pub fn list(archive: &Path) -> Result<Vec<ListEntry>> {
    let file = File::open(archive)?;
    list_reader(file)
}
