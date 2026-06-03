use std::path::{Path, PathBuf};

use walkdir::WalkDir;

use crate::error::{EpaxError, Result};

/// A single file to place into an archive.
pub struct Entry {
    /// Path on disk to read from.
    pub path: PathBuf,
    /// Name to store inside the archive (always uses `/` separators).
    pub arcname: String,
}

/// Expand the user's input paths into a flat list of files. A directory input
/// is walked recursively and its entries are stored relative to the directory's
/// parent, so `epax compress out.zip ./src` yields `src/main.rs`, etc.
///
/// Only regular files are stored; empty directories are not preserved in v1.
pub fn collect_inputs(inputs: &[PathBuf]) -> Result<Vec<Entry>> {
    if inputs.is_empty() {
        return Err(EpaxError::NoInputs);
    }

    let mut entries = Vec::new();
    for input in inputs {
        if !input.exists() {
            return Err(EpaxError::MissingInput(input.clone()));
        }

        // Base directory that arcnames are computed relative to.
        let base = input.parent().unwrap_or_else(|| Path::new(""));

        if input.is_dir() {
            for dent in WalkDir::new(input).follow_links(false) {
                let dent = dent.map_err(|e| EpaxError::Backend(e.to_string()))?;
                if dent.file_type().is_file() {
                    let arcname = to_arcname(dent.path(), base)?;
                    entries.push(Entry {
                        path: dent.path().to_path_buf(),
                        arcname,
                    });
                }
            }
        } else {
            let arcname = to_arcname(input, base)?;
            entries.push(Entry {
                path: input.clone(),
                arcname,
            });
        }
    }
    Ok(entries)
}

/// Compute a forward-slash arcname for `path` relative to `base`.
fn to_arcname(path: &Path, base: &Path) -> Result<String> {
    let rel = path.strip_prefix(base).unwrap_or(path);
    let parts: Vec<String> = rel
        .components()
        .filter_map(|c| match c {
            std::path::Component::Normal(p) => Some(p.to_string_lossy().to_string()),
            _ => None,
        })
        .collect();
    if parts.is_empty() {
        return Err(EpaxError::Backend(format!(
            "cannot derive archive name for {}",
            path.display()
        )));
    }
    Ok(parts.join("/"))
}
