use std::path::Path;

use crate::backends::{ListEntry, sevenz, streamc, tar, zip};
use crate::error::Result;
use crate::format::{Format, name_implies_tar};

/// Run a list operation, returning the archive's entries.
pub fn run(archive: &Path, format: &Option<String>) -> Result<Vec<ListEntry>> {
    let fmt = match format {
        Some(name) => Format::from_name(name)?,
        None => Format::detect_from_path(archive)?,
    };
    match fmt {
        Format::Zip => zip::list(archive),
        Format::SevenZ => sevenz::list(archive),
        Format::Tar => tar::list(archive),
        #[cfg(feature = "rar")]
        Format::Rar => crate::backends::rar::list(archive),
        #[cfg(not(feature = "rar"))]
        Format::Rar => Err(crate::error::EpaxError::RarUnavailable),
        Format::Gz | Format::Bz2 | Format::Zst => {
            let is_tar = name_implies_tar(archive);
            streamc::list(fmt, archive, is_tar)
        }
    }
}
