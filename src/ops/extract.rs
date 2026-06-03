use std::path::Path;

use crate::backends::{sevenz, streamc, tar, zip};
use crate::error::{EpaxError, Result};
use crate::format::{Format, name_implies_tar, strip_archive_suffix};

/// Resolve the format for an extract/list operation.
/// Falls back to magic-byte detection when the extension is not recognized.
fn resolve_format(archive: &Path, format: &Option<String>) -> Result<Format> {
    if let Some(name) = format {
        return Format::from_name(name);
    }
    match Format::detect_from_path(archive) {
        Ok(fmt) => return Ok(fmt),
        Err(EpaxError::UnknownFormat(_)) => {}
        Err(e) => return Err(e),
    }
    // Extension unrecognized — try magic bytes.
    match Format::detect_from_magic(archive) {
        Ok(Some(fmt)) => Ok(fmt),
        Ok(None) => Err(EpaxError::UnknownFormat(
            archive
                .file_name()
                .map(|s| s.to_string_lossy().into_owned())
                .unwrap_or_default(),
        )),
        Err(e) => Err(EpaxError::Io(e)),
    }
}

/// Derive output directory from the archive path when -o is not given.
/// `backup.zip` → `backup`, `release.tar.gz` → `release`.
fn derive_output_dir(archive: &Path) -> std::path::PathBuf {
    let name = archive
        .file_name()
        .map(|s| s.to_string_lossy().into_owned())
        .unwrap_or_else(|| "extracted".to_string());
    let stripped = strip_archive_suffix(&name).to_string();
    // If stripping produced empty string, fall back to "extracted".
    if stripped.is_empty() {
        std::path::PathBuf::from("extracted")
    } else {
        std::path::PathBuf::from(stripped)
    }
}

/// Run an extract operation.
pub fn run(archive: &Path, dest: Option<&Path>, format: &Option<String>, verbose: bool) -> Result<()> {
    let output = match dest {
        Some(d) => d.to_path_buf(),
        None => derive_output_dir(archive),
    };

    let fmt = resolve_format(archive, format)?;
    match fmt {
        Format::Zip => zip::extract(archive, &output, verbose),
        Format::SevenZ => sevenz::extract(archive, &output, verbose),
        Format::Tar => tar::extract(archive, &output, verbose),
        #[cfg(feature = "rar")]
        Format::Rar => crate::backends::rar::extract(archive, &output, verbose),
        #[cfg(not(feature = "rar"))]
        Format::Rar => Err(crate::error::EpaxError::RarUnavailable),
        Format::Gz | Format::Bz2 | Format::Zst => {
            let is_tar = name_implies_tar(archive);
            streamc::extract(fmt, archive, &output, verbose, is_tar)
        }
    }
}
