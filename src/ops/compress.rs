use std::path::{Path, PathBuf};

use crate::backends::{sevenz, streamc, tar, zip};
use crate::collect::collect_inputs;
use crate::error::{EpaxError, Result};
use crate::format::{Format, name_implies_tar};

/// Resolve the format for a compress operation, honoring an explicit override.
fn resolve_format(output: &Path, format: &Option<String>) -> Result<Format> {
    match format {
        Some(name) => Format::from_name(name),
        None => Format::detect_from_path(output),
    }
}

/// Run a compress operation.
pub fn run(
    output: &Path,
    inputs: &[PathBuf],
    format: &Option<String>,
    level: Option<i32>,
    verbose: bool,
) -> Result<()> {
    let fmt = resolve_format(output, format)?;
    if !fmt.can_compress() {
        return Err(EpaxError::CompressNotSupported(fmt.label()));
    }

    let entries = collect_inputs(inputs)?;

    match fmt {
        Format::Zip => zip::compress(output, &entries, level, verbose),
        Format::SevenZ => sevenz::compress(output, &entries, level, verbose),
        Format::Tar => tar::compress(output, &entries, verbose),
        Format::Gz | Format::Bz2 | Format::Zst => {
            // Use a tar container when packing a directory, multiple inputs, or
            // when the name explicitly asks for it (.tar.gz / .tgz / ...).
            // A single plain file with a bare .gz/.bz2/.zst name is compressed raw.
            let multi = entries.len() > 1;
            let tar_mode = multi || name_implies_tar(output);
            streamc::compress(fmt, output, &entries, level, verbose, tar_mode)
        }
        Format::Rar => Err(EpaxError::CompressNotSupported(fmt.label())),
    }
}
