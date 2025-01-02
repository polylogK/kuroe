use crate::language::{default_languages, CustomLang, Language};
use anyhow::{bail, Result};
use regex::Regex;
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

pub(crate) fn make_languages(
    custom_language: &Vec<String>,
) -> Result<Vec<Box<dyn Language + 'static>>> {
    match custom_language.len() {
        0 => Ok(default_languages()),
        1 => {
            bail!("invalid custom language: {:?}", custom_language);
        }
        _ => {
            let mut langs = default_languages();
            let custom_lang = CustomLang::new(
                Regex::new(&custom_language[0])?,
                custom_language[1..].to_vec(),
            )?;
            langs.insert(0, Box::new(custom_lang));
            Ok(langs)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_find_files() {
        let files = find_files(Path::new("./src/main.rs"), false).unwrap();
        assert_eq!(files.len(), 1);

        let files = find_files(Path::new("./example"), true).unwrap();
        assert!(files.len() > 1);
    }

    #[test]
    fn test_make_languages() {
        let default_langs = default_languages();

        let langs = make_languages(&Vec::new()).unwrap();
        assert_eq!(langs.len(), default_langs.len());

        let langs = make_languages(&vec![
            "cpp".to_string(),
            "g++ %(target)".to_string(),
            "./a.out".to_string(),
        ])
        .unwrap();
        assert_eq!(langs.len(), default_langs.len() + 1);

        let langs = make_languages(&vec!["invalid".to_string()]);
        assert!(langs.is_err());
    }
}
