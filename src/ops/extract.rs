use std::path::Path;

use crate::backends::{sevenz, streamc, tar, zip};
use crate::error::{EpaxError, Result};
use crate::format::{Format, name_implies_tar};

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

/// Run an extract operation.
pub fn run(archive: &Path, dest: &Path, format: &Option<String>, verbose: bool) -> Result<()> {
    let fmt = resolve_format(archive, format)?;
    match fmt {
        Format::Zip => zip::extract(archive, dest, verbose),
        Format::SevenZ => sevenz::extract(archive, dest, verbose),
        Format::Tar => tar::extract(archive, dest, verbose),
        #[cfg(feature = "rar")]
        Format::Rar => crate::backends::rar::extract(archive, dest, verbose),
        #[cfg(not(feature = "rar"))]
        Format::Rar => Err(crate::error::EpaxError::RarUnavailable),
        Format::Gz | Format::Bz2 | Format::Zst => {
            // For magic-detected gz/bz2/zst we don't know if there's a tar inside;
            // try to detect from the name, then treat as bare stream by default.
            let is_tar = name_implies_tar(archive);
            streamc::extract(fmt, archive, dest, verbose, is_tar)
        }
    }
}
