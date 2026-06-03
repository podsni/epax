use std::path::PathBuf;

/// Crate-wide result type.
pub type Result<T> = std::result::Result<T, EpaxError>;

/// All errors surfaced by epax.
#[derive(Debug, thiserror::Error)]
pub enum EpaxError {
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),

    /// Could not determine the archive format from the file name.
    #[error("could not determine format for '{0}' — pass --format to set it explicitly")]
    UnknownFormat(String),

    /// User passed an unrecognized value to --format.
    #[error("unknown format '{0}' (expected one of: zip, 7z, gz, bz2, zst, tar, rar)")]
    BadFormatName(String),

    /// Creating this format is impossible/unsupported (e.g. RAR).
    #[error("creating {0} archives is not supported (the RAR format is proprietary; epax can only extract it)")]
    CompressNotSupported(&'static str),

    /// RAR extraction was requested but the `rar` feature was not compiled in.
    #[cfg(not(feature = "rar"))]
    #[error("this build of epax has no RAR support (rebuilt without the 'rar' feature)")]
    RarUnavailable,

    /// An archive entry path tried to escape the destination directory.
    #[error("refusing to extract unsafe path '{0}' (path traversal attempt)")]
    UnsafePath(String),

    /// No input paths were given to compress.
    #[error("no input files given to compress")]
    NoInputs,

    /// An input path does not exist.
    #[error("input path does not exist: {0}")]
    MissingInput(PathBuf),

    /// Backend-specific failure carrying a descriptive message.
    #[error("{0}")]
    Backend(String),
}

impl EpaxError {
    /// Process exit code associated with this error.
    pub fn exit_code(&self) -> i32 {
        match self {
            EpaxError::CompressNotSupported(_) => 2,
            _ => 1,
        }
    }
}
