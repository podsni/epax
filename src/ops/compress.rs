use std::io::{self, Write as _};
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
    if let Some(name) = format {
        let fmt = Format::from_name(name)?;
        let output_is_archive = Format::detect_from_path(output).is_ok();
        if !output_is_archive && inputs.is_empty() {
            let stem = output
                .file_stem()
                .map(|s| s.to_string_lossy().into_owned())
                .filter(|s| !s.is_empty() && s != ".")
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

    match Format::detect_from_path(output) {
        Ok(fmt) => return Ok((output.to_path_buf(), inputs.to_vec(), fmt)),
        Err(EpaxError::UnknownFormat(_)) => {}
        Err(e) => return Err(e),
    }

    // Auto-output mode.
    let mut all_inputs = vec![output.to_path_buf()];
    all_inputs.extend_from_slice(inputs);

    let stem = all_inputs[0]
        .file_stem()
        .map(|s| s.to_string_lossy().into_owned())
        .filter(|s| !s.is_empty() && s != ".")
        .unwrap_or_else(|| "archive".to_string());

    let auto_output = PathBuf::from(format!("{stem}.zip"));
    Ok((auto_output, all_inputs, Format::Zip))
}

/// Core compress operation.  Returns the actual output path used.
pub fn run(output: &Path, inputs: &[PathBuf], format: &Option<String>, level: Option<i32>, verbose: bool) -> Result<PathBuf> {
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

/// Prompt for a single line of input; return trimmed string.
fn prompt(msg: &str, default: &str) -> String {
    print!("{} [{}]: ", msg, default);
    io::stdout().flush().ok();
    let mut line = String::new();
    io::stdin().read_line(&mut line).ok();
    let t = line.trim().to_string();
    if t.is_empty() { default.to_string() } else { t }
}

/// Interactive compress guide – collects inputs, format and options from the
/// user via stdin prompts, then runs the compress operation.
pub fn run_interactive(output: Option<&Path>, inputs: &[PathBuf], format: &Option<String>, level: Option<i32>, verbose: bool) -> Result<PathBuf> {
    println!("\n── epax interactive compress ──");
    println!("(press Enter on empty line to finish adding files)\n");

    // Collect files — pre-fill from CLI arguments
    let mut all_inputs: Vec<PathBuf> = inputs.to_vec();

    loop {
        let existing = if all_inputs.is_empty() { "" } else { "" };
        print!("  add file/dir{existing}: ");
        io::stdout().flush().ok();
        let mut line = String::new();
        io::stdin().read_line(&mut line).ok();
        let t = line.trim().to_string();
        if t.is_empty() && !all_inputs.is_empty() {
            break;
        }
        if t.is_empty() {
            continue;
        }
        all_inputs.push(PathBuf::from(&t));
    }

    if all_inputs.is_empty() {
        return Err(EpaxError::Backend("no input files given, aborting".to_string()));
    }

    // Show what was collected
    println!("\n  inputs ({}):", all_inputs.len());
    for inp in &all_inputs {
        println!("    {}", inp.display());
    }

    // Format
    let fmt_str = if let Some(f) = format {
        f.clone()
    } else {
        prompt("  output format", "zip")
    };
    let fmt = Format::from_name(&fmt_str).map_err(|_| EpaxError::Backend(format!("invalid format '{fmt_str}'")))?;

    // Output path — use pre-filled output if given, else auto-generate from first input
    let default_out = match output {
        Some(p) => p.to_string_lossy().into_owned(),
        None => {
            let stem = all_inputs[0]
                .file_stem()
                .map(|s| s.to_string_lossy().into_owned())
                .filter(|s| !s.is_empty() && s != ".")
                .unwrap_or_else(|| "archive".to_string());
            format!("{stem}.{}", match fmt {
                Format::Zip => "zip",
                Format::SevenZ => "7z",
                Format::Tar => "tar",
                Format::Gz => "tar.gz",
                Format::Bz2 => "tar.bz2",
                Format::Zst => "tar.zst",
                Format::Rar => "rar",
            })
        }
    };
    let out_str = prompt("  output path", &default_out);
    let out_path = PathBuf::from(&out_str);

    // Level
    let lvl: Option<i32> = level.or_else(|| {
        let l_str = prompt("  compression level (default: format default)", "auto");
        if l_str == "auto" { None } else { l_str.parse::<i32>().ok() }
    });

    // Verbose
    let v = if verbose { true } else {
        prompt("  verbose? (y/n)", "n").eq_ignore_ascii_case("y")
    };

    // Summary + confirm
    println!("\n  ─── summary ───");
    println!("  format:  {}", fmt.label());
    println!("  output:  {}", out_path.display());
    println!("  inputs:  {}", all_inputs.len());
    if let Some(l) = lvl { println!("  level:   {l}"); }
    println!("  ───────────────");

    let ok = prompt("  proceed?", "Y");
    if !ok.eq_ignore_ascii_case("y") && !ok.eq_ignore_ascii_case("yes") && ok != "Y" {
        println!("  aborted.");
        return Err(EpaxError::Backend("user aborted".to_string()));
    }

    // Execute
    let result = run(&out_path, &all_inputs, &Some(fmt_str.clone()), lvl, v)?;
    println!("  created {}", result.display());
    Ok(result)
}
