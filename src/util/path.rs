use std::path::{Component, Path, PathBuf};

use crate::error::{EpaxError, Result};

/// Join an archive entry path onto a destination directory, guaranteeing the
/// result stays inside `dest`. Defends against zip-slip / path-traversal:
/// absolute paths, drive prefixes, and `..` escapes are rejected.
pub fn sanitize_entry_path(dest: &Path, entry: &Path) -> Result<PathBuf> {
    let mut out = dest.to_path_buf();
    let mut depth: i32 = 0;

    for comp in entry.components() {
        match comp {
            Component::Normal(part) => {
                out.push(part);
                depth += 1;
            }
            Component::CurDir => {}
            Component::ParentDir => {
                if depth == 0 {
                    return Err(EpaxError::UnsafePath(entry.display().to_string()));
                }
                depth -= 1;
                out.pop();
            }
            // Absolute roots and Windows prefixes must never appear in an entry.
            Component::RootDir | Component::Prefix(_) => {
                return Err(EpaxError::UnsafePath(entry.display().to_string()));
            }
        }
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn safe_relative_path_ok() {
        let p = sanitize_entry_path(Path::new("/out"), Path::new("a/b/c.txt")).unwrap();
        assert_eq!(p, PathBuf::from("/out/a/b/c.txt"));
    }

    #[test]
    fn interior_parent_dir_ok() {
        let p = sanitize_entry_path(Path::new("/out"), Path::new("a/b/../c.txt")).unwrap();
        assert_eq!(p, PathBuf::from("/out/a/c.txt"));
    }

    #[test]
    fn escaping_parent_dir_rejected() {
        assert!(sanitize_entry_path(Path::new("/out"), Path::new("../evil")).is_err());
        assert!(sanitize_entry_path(Path::new("/out"), Path::new("a/../../evil")).is_err());
    }

    #[test]
    fn absolute_entry_rejected() {
        assert!(sanitize_entry_path(Path::new("/out"), Path::new("/etc/passwd")).is_err());
    }
}
