use std::path::{Path, PathBuf};

use crate::backends::{sevenz, streamc, tar, zip};
use crate::collect::collect_inputs;
use crate::error::{EpaxError, Result};
use crate::format::{Format, name_implies_tar};

/// Resolve output path and format for a compress operation.
///
/// When `output` has no recognized archive extension and no `--format` is
/// given, treat `output` as an additional input and auto-generate the output
/// name: stem of the first input + `.zip`.  Returns `(actual_output, inputs)`.
pub fn resolve_output(
    output: &Path,
    inputs: &[PathBuf],
    format: &Option<String>,
) -> Result<(PathBuf, Vec<PathBuf>, Format)> {
    // If --format was given we know the target format.
    // Still auto-generate the output name when: the given "output" path has no
    // recognized archive extension AND there are no extra inputs — meaning the
    // user wrote `epax c doc.pdf --format 7z` intending doc.pdf as the input.
    if let Some(name) = format {
        let fmt = Format::from_name(name)?;
        let output_is_archive = Format::detect_from_path(output).is_ok();
        if !output_is_archive && inputs.is_empty() {
            let stem = output
                .file_stem()
                .map(|s| s.to_string_lossy().into_owned())
                .unwrap_or_else(|| "archive".to_string());
            let ext = match fmt {
                Format::Zip => "zip",
                Format::SevenZ => "7z",
                Format::Gz => "tar.gz",
                Format::Bz2 => "tar.bz2",
                Format::Zst => "tar.zst",
                Format::Tar => "tar",
                Format::Rar => "rar",
            };
            let auto_output = output.with_file_name(format!("{stem}.{ext}"));
            return Ok((auto_output, vec![output.to_path_buf()], fmt));
        }
        return Ok((output.to_path_buf(), inputs.to_vec(), fmt));
    }

    // Try to detect from the given output path.
    match Format::detect_from_path(output) {
        Ok(fmt) => return Ok((output.to_path_buf(), inputs.to_vec(), fmt)),
        Err(EpaxError::UnknownFormat(_)) => {}
        Err(e) => return Err(e),
    }

    // Output path has no recognized extension — treat it as the first input
    // and auto-generate an output name.
    let mut all_inputs = vec![output.to_path_buf()];
    all_inputs.extend_from_slice(inputs);

    // Stem of the first input becomes the archive base name.
    let stem = all_inputs[0]
        .file_stem()
        .map(|s| s.to_string_lossy().into_owned())
        .unwrap_or_else(|| "archive".to_string());

    let auto_output = PathBuf::from(format!("{stem}.zip"));
    Ok((auto_output, all_inputs, Format::Zip))
}

/// Run a compress operation.  Returns the actual output path used.
pub fn run(
    output: &Path,
    inputs: &[PathBuf],
    format: &Option<String>,
    level: Option<i32>,
    verbose: bool,
) -> Result<PathBuf> {
    let (actual_output, all_inputs, fmt) = resolve_output(output, inputs, format)?;

    if !fmt.can_compress() {
        return Err(EpaxError::CompressNotSupported(fmt.label()));
    }

    let entries = collect_inputs(&all_inputs)?;

    match fmt {
        Format::Zip => zip::compress(&actual_output, &entries, level, verbose)?,
        Format::SevenZ => sevenz::compress(&actual_output, &entries, level, verbose)?,
        Format::Tar => tar::compress(&actual_output, &entries, verbose)?,
        Format::Gz | Format::Bz2 | Format::Zst => {
            let multi = entries.len() > 1;
            let tar_mode = multi || name_implies_tar(&actual_output);
            streamc::compress(fmt, &actual_output, &entries, level, verbose, tar_mode)?;
        }
        Format::Rar => return Err(EpaxError::CompressNotSupported(fmt.label())),
    }

    Ok(actual_output)
}
