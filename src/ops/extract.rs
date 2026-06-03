use std::path::Path;

use crate::backends::{sevenz, streamc, tar, zip};
use crate::error::Result;
use crate::format::{Format, name_implies_tar};

/// Resolve the format for an extract/list operation.
fn resolve_format(archive: &Path, format: &Option<String>) -> Result<Format> {
    match format {
        Some(name) => Format::from_name(name),
        None => Format::detect_from_path(archive),
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
            let is_tar = name_implies_tar(archive);
            streamc::extract(fmt, archive, dest, verbose, is_tar)
        }
    }
}
