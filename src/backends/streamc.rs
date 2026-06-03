use std::fs::File;
use std::io::{self, Read, Write};
use std::path::Path;

use crate::backends::tar as tarb;
use crate::backends::{ListEntry, clamp_level};
use crate::collect::Entry;
use crate::error::{EpaxError, Result};
use crate::format::{Format, strip_compression_suffix};

/// Build a boxed streaming encoder for one of the single-stream formats.
/// Dropping the returned writer finalizes the compressed stream (writes the
/// trailer/epilogue), so callers must let it drop after the last write.
fn make_encoder(format: Format, file: File, level: Option<i32>) -> Result<Box<dyn Write>> {
    let enc: Box<dyn Write> = match format {
        Format::Gz => {
            let l = clamp_level(level, 0, 9, 6) as u32;
            Box::new(flate2::write::GzEncoder::new(
                file,
                flate2::Compression::new(l),
            ))
        }
        Format::Bz2 => {
            let l = clamp_level(level, 1, 9, 6) as u32;
            Box::new(bzip2::write::BzEncoder::new(file, bzip2::Compression::new(l)))
        }
        Format::Zst => {
            let l = clamp_level(level, 1, 22, 3);
            Box::new(zstd::stream::write::Encoder::new(file, l)?.auto_finish())
        }
        other => {
            return Err(EpaxError::Backend(format!(
                "{} is not a stream format",
                other.label()
            )));
        }
    };
    Ok(enc)
}

/// Build a boxed streaming decoder for one of the single-stream formats.
fn make_decoder(format: Format, file: File) -> Result<Box<dyn Read>> {
    let dec: Box<dyn Read> = match format {
        Format::Gz => Box::new(flate2::read::GzDecoder::new(file)),
        Format::Bz2 => Box::new(bzip2::read::BzDecoder::new(file)),
        Format::Zst => Box::new(zstd::stream::read::Decoder::new(file)?),
        other => {
            return Err(EpaxError::Backend(format!(
                "{} is not a stream format",
                other.label()
            )));
        }
    };
    Ok(dec)
}

fn file_name(path: &Path) -> String {
    path.file_name()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_else(|| "output".to_string())
}

/// Compress with a stream format. When `tar_mode` is true the inputs are packed
/// into a tar stream first (`.tar.gz` etc.); otherwise the single input file's
/// bytes are compressed directly (bare `.gz` / `.bz2` / `.zst`).
pub fn compress(
    format: Format,
    output: &Path,
    entries: &[Entry],
    level: Option<i32>,
    verbose: bool,
    tar_mode: bool,
) -> Result<()> {
    let file = File::create(output)?;
    let encoder = make_encoder(format, file, level)?;

    if tar_mode {
        // write_entries returns the boxed encoder; dropping it flushes + finalizes.
        let encoder = tarb::write_entries(encoder, entries, verbose)?;
        drop(encoder);
    } else {
        let entry = &entries[0];
        if verbose {
            println!("  adding: {}", entry.arcname);
        }
        let mut encoder = encoder;
        let mut input = File::open(&entry.path)?;
        io::copy(&mut input, &mut encoder)?;
        encoder.flush()?;
        drop(encoder);
    }
    Ok(())
}

/// Extract a stream format. When `is_tar` the decompressed stream is treated as
/// a tar archive and unpacked into `dest`; otherwise a single decompressed file
/// is written into `dest` (with the compression suffix stripped from its name).
pub fn extract(
    format: Format,
    archive: &Path,
    dest: &Path,
    verbose: bool,
    is_tar: bool,
) -> Result<()> {
    std::fs::create_dir_all(dest)?;
    let file = File::open(archive)?;
    let mut decoder = make_decoder(format, file)?;

    if is_tar {
        tarb::extract_reader(decoder, dest, verbose)?;
    } else {
        let inner = strip_compression_suffix(&file_name(archive));
        let out_path = dest.join(inner);
        if verbose {
            println!(" extracting: {}", out_path.display());
        }
        let mut out = File::create(&out_path)?;
        io::copy(&mut decoder, &mut out)?;
    }
    Ok(())
}

/// List a stream format's contents.
pub fn list(format: Format, archive: &Path, is_tar: bool) -> Result<Vec<ListEntry>> {
    let file = File::open(archive)?;
    let decoder = make_decoder(format, file)?;
    if is_tar {
        tarb::list_reader(decoder)
    } else {
        // A bare single-stream file holds exactly one logical entry. Its size is
        // only known after full decompression, so report it as unknown (0).
        Ok(vec![ListEntry {
            name: strip_compression_suffix(&file_name(archive)),
            size: 0,
        }])
    }
}
