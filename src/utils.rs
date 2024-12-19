use anyhow::Result;
use std::fs;
use std::path::{Path, PathBuf};

pub(crate) fn find_files(base: &Path, recursive: bool) -> Result<Vec<PathBuf>> {
    let mut generators = Vec::new();
    if base.is_file() {
        generators.push(base.to_path_buf());
    } else if base.is_dir() {
        for entry in fs::read_dir(base)? {
            let path = entry?.path();

            if path.is_file() {
                generators.push(path);
            } else if path.is_dir() && recursive {
                let mut sub_files = find_files(&path, recursive)?;
                generators.append(&mut sub_files);
            }
        }
    }

    Ok(generators)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_find_files() {
        let files = find_files(Path::new("./example"), false).unwrap();
        assert_eq!(files.len(), 0);

        let files = find_files(Path::new("./example"), true).unwrap();
        assert!(files.len() > 1);
    }
}
