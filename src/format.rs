use std::io::{self, Read};
use std::path::Path;

use crate::error::{EpaxError, Result};

/// Archive / compression formats epax understands.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Format {
    Zip,
    SevenZ,
    Gz,
    Bz2,
    Zst,
    Tar,
    Rar,
}

impl Format {
    /// Human-readable label used in messages.
    pub fn label(self) -> &'static str {
        match self {
            Format::Zip => "zip",
            Format::SevenZ => "7z",
            Format::Gz => "gzip",
            Format::Bz2 => "bzip2",
            Format::Zst => "zstd",
            Format::Tar => "tar",
            Format::Rar => "rar",
        }
    }

    /// Whether this format can be created. RAR is extract-only.
    pub fn can_compress(self) -> bool {
        !matches!(self, Format::Rar)
    }

    /// Parse a `--format` value.
    pub fn from_name(name: &str) -> Result<Format> {
        let f = match name.to_ascii_lowercase().as_str() {
            "zip" => Format::Zip,
            "7z" | "7zip" | "sevenz" => Format::SevenZ,
            "gz" | "gzip" => Format::Gz,
            "bz2" | "bzip2" => Format::Bz2,
            "zst" | "zstd" => Format::Zst,
            "tar" => Format::Tar,
            "rar" => Format::Rar,
            other => return Err(EpaxError::BadFormatName(other.to_string())),
        };
        Ok(f)
    }

    /// Detect a format from a file name. Double extensions like `.tar.gz` and
    /// short forms like `.tgz` are recognized and mapped to the underlying
    /// stream compressor (with the tar container implied).
    pub fn detect_from_path(path: &Path) -> Result<Format> {
        let name = path
            .file_name()
            .map(|s| s.to_string_lossy().to_ascii_lowercase())
            .unwrap_or_default();

        // Longest / most specific suffixes first.
        let f = if name.ends_with(".tar.gz") || name.ends_with(".tgz") {
            Format::Gz
        } else if name.ends_with(".tar.bz2") || name.ends_with(".tbz2") || name.ends_with(".tbz") {
            Format::Bz2
        } else if name.ends_with(".tar.zst") || name.ends_with(".tzst") {
            Format::Zst
        } else if name.ends_with(".zip") {
            Format::Zip
        } else if name.ends_with(".7z") {
            Format::SevenZ
        } else if name.ends_with(".rar") {
            Format::Rar
        } else if name.ends_with(".tar") {
            Format::Tar
        } else if name.ends_with(".gz") {
            Format::Gz
        } else if name.ends_with(".bz2") {
            Format::Bz2
        } else if name.ends_with(".zst") {
            Format::Zst
        } else {
            return Err(EpaxError::UnknownFormat(name));
        };
        Ok(f)
    }

    /// Sniff the first few bytes of a file and return the format, if recognized.
    /// Returns None when the magic bytes don't match any known format.
    pub fn detect_from_magic(path: &Path) -> io::Result<Option<Format>> {
        let mut file = std::fs::File::open(path)?;
        let mut buf = [0u8; 8];
        let n = file.read(&mut buf)?;
        let b = &buf[..n];

        let fmt = if b.starts_with(&[0x50, 0x4B, 0x03, 0x04])
            || b.starts_with(&[0x50, 0x4B, 0x05, 0x06])
            || b.starts_with(&[0x50, 0x4B, 0x07, 0x08])
        {
            Some(Format::Zip)
        } else if b.starts_with(b"7z\xbc\xaf\x27\x1c") {
            Some(Format::SevenZ)
        } else if b.starts_with(&[0x1f, 0x8b]) {
            Some(Format::Gz)
        } else if b.starts_with(&[0x42, 0x5a, 0x68]) {
            Some(Format::Bz2)
        } else if b.starts_with(&[0x28, 0xb5, 0x2f, 0xfd]) {
            Some(Format::Zst)
        } else if b.len() >= 5 && &b[..5] == b"Rar!!" {
            Some(Format::Rar)
        } else {
            None
        };
        Ok(fmt)
    }
}

/// Does this file name imply a tar container around the compressed stream?
/// True for `.tar.gz`, `.tgz`, `.tar.zst`, `.tzst`, etc. False for a bare
/// `.gz` / `.bz2` / `.zst`.
pub fn name_implies_tar(path: &Path) -> bool {
    let name = path
        .file_name()
        .map(|s| s.to_string_lossy().to_ascii_lowercase())
        .unwrap_or_default();
    name.ends_with(".tar.gz")
        || name.ends_with(".tgz")
        || name.ends_with(".tar.bz2")
        || name.ends_with(".tbz2")
        || name.ends_with(".tbz")
        || name.ends_with(".tar.zst")
        || name.ends_with(".tzst")
}

/// Strip a known compression suffix to recover the inner file name when
/// decompressing a bare single-file stream (e.g. `notes.txt.gz` -> `notes.txt`,
/// `data.tgz` -> `data.tar`).
pub fn strip_compression_suffix(name: &str) -> String {
    let lower = name.to_ascii_lowercase();
    for (suffix, replacement) in [
        (".tar.gz", ".tar"),
        (".tar.bz2", ".tar"),
        (".tar.zst", ".tar"),
        (".tgz", ".tar"),
        (".tbz2", ".tar"),
        (".tbz", ".tar"),
        (".tzst", ".tar"),
    ] {
        if lower.ends_with(suffix) {
            return format!("{}{}", &name[..name.len() - suffix.len()], replacement);
        }
    }
    for suffix in [".gz", ".bz2", ".zst"] {
        if lower.ends_with(suffix) {
            return name[..name.len() - suffix.len()].to_string();
        }
    }
    format!("{name}.out")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn detect(s: &str) -> Format {
        Format::detect_from_path(&PathBuf::from(s)).unwrap()
    }

    #[test]
    fn detects_simple_extensions() {
        assert_eq!(detect("a.zip"), Format::Zip);
        assert_eq!(detect("a.7z"), Format::SevenZ);
        assert_eq!(detect("a.rar"), Format::Rar);
        assert_eq!(detect("a.tar"), Format::Tar);
        assert_eq!(detect("a.gz"), Format::Gz);
        assert_eq!(detect("a.bz2"), Format::Bz2);
        assert_eq!(detect("a.zst"), Format::Zst);
    }

    #[test]
    fn detects_double_and_short_extensions() {
        assert_eq!(detect("a.tar.gz"), Format::Gz);
        assert_eq!(detect("a.tgz"), Format::Gz);
        assert_eq!(detect("a.tar.bz2"), Format::Bz2);
        assert_eq!(detect("a.tbz2"), Format::Bz2);
        assert_eq!(detect("a.tar.zst"), Format::Zst);
        assert_eq!(detect("a.tzst"), Format::Zst);
    }

    #[test]
    fn detection_is_case_insensitive() {
        assert_eq!(detect("PHOTO.ZIP"), Format::Zip);
        assert_eq!(detect("Backup.Tar.Gz"), Format::Gz);
    }

    #[test]
    fn unknown_extension_errors() {
        assert!(Format::detect_from_path(&PathBuf::from("a.txt")).is_err());
        assert!(Format::detect_from_path(&PathBuf::from("noext")).is_err());
    }

    #[test]
    fn name_implies_tar_works() {
        assert!(name_implies_tar(&PathBuf::from("a.tar.gz")));
        assert!(name_implies_tar(&PathBuf::from("a.tgz")));
        assert!(name_implies_tar(&PathBuf::from("a.tar.zst")));
        assert!(!name_implies_tar(&PathBuf::from("a.gz")));
        assert!(!name_implies_tar(&PathBuf::from("a.zst")));
    }

    #[test]
    fn strip_suffix_works() {
        assert_eq!(strip_compression_suffix("notes.txt.gz"), "notes.txt");
        assert_eq!(strip_compression_suffix("data.tar.zst"), "data.tar");
        assert_eq!(strip_compression_suffix("data.tgz"), "data.tar");
        assert_eq!(strip_compression_suffix("blob.bz2"), "blob");
    }

    #[test]
    fn rar_cannot_compress() {
        assert!(!Format::Rar.can_compress());
        assert!(Format::Zip.can_compress());
    }
}
