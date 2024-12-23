use crate::language::{detect_language, Clang, Cpp, CustomLang, Language, Python, Txt};
use crate::utils::find_files;
use anyhow::{Context, Result};
use clap::Args;
use regex::Regex;
use std::fs::{create_dir_all, File};
use std::path::{Path, PathBuf};
use std::process::Stdio;
use std::time::Duration;
use tempfile::TempDir;

#[derive(Debug, Args)]
pub(super) struct GenerateArgs {
    /// directory containing the generator or path to the generator
    #[arg(value_name = "GENERATOR", required = true)]
    generators: Vec<PathBuf>,

    /// recursively search for generators
    #[arg(short, long, default_value_t = false)]
    recursive: bool,

    ///
    #[arg(short, long, default_value = "./testcases/input")]
    outdir: PathBuf,

    /// number of generation per generator. Specifying by filename has higher priority
    #[arg(short = 'n', long, default_value_t = 1
    , value_parser = clap::value_parser!(u32).range(1..))]
    count: u32,

    /// seed, seed+1, seed+2, ..., seed+(n-1)
    #[arg(short, long, default_value_t = 0, required = false
    , value_parser = clap::value_parser!(u32).range(0..))]
    seed: u32,

    /// COMMAND[0:-1] are the compile commands. COMMAND[-1] is execute command
    #[arg(
        short,
        long,
        value_name = "<EXT> <COMMAND>...",
        required = false,
        value_delimiter = ','
    )]
    language: Vec<String>,
}

#[derive(Debug)]
struct GenFileInfo {
    name: String,
    count: Option<u32>,
    ext: String,
}

impl From<&Path> for GenFileInfo {
    /// hoge.{count}.ext を解釈する
    /// count が u32 としてパースできる場合は name = "hoge"
    /// count が u32 としてパースできない場合は name = "hoge.{count}"
    fn from(path: &Path) -> Self {
        let ext = path.extension().unwrap().to_string_lossy().to_string();
        let stem = path.file_stem().unwrap().to_string_lossy().to_string();

        // count の解決
        let parts: Vec<&str> = stem.rsplitn(2, '.').collect();
        let count = parts.get(0).and_then(|s| s.parse::<u32>().ok());
        let name = if count.is_some() {
            parts.get(1).map(|s| s.to_string()).unwrap()
        } else {
            stem
        };

        GenFileInfo { name, count, ext }
    }
}

/// 生成されたテストケースへのパスを返す
fn generate(
    target: &Path,
    outdir: &Path,
    count: u32,
    seed: u32,
    langs: &Vec<Box<dyn Language>>,
) -> Result<Vec<PathBuf>> {
    let info = GenFileInfo::from(target);
    let target = target.canonicalize()?;
    let lang = detect_language(&info.ext, langs)?;

    // compile
    let dir = TempDir::new()?;
    for step in lang.compile(&target) {
        step.execute(&dir, Stdio::null(), Stdio::null(), Duration::from_secs(10))?;
    }

    // generate
    let count = info.count.unwrap_or(count);
    let mut generated_cases = Vec::new();
    for i in 0..count {
        let output_name = format!("{}_{:03}.in", &info.name, i);
        let output_path = outdir.join(output_name);
        let output = File::create(&output_path).unwrap();

        lang.run(&target, seed + i as u32)
            .execute(&dir, Stdio::null(), output, Duration::from_secs(10))
            .with_context(|| format!("failed to generate {:?} at seed = {:?}", target, seed + i))?;

        generated_cases.push(output_path.to_path_buf());
    }

    Ok(generated_cases)
}

pub(super) fn root(args: GenerateArgs) -> Result<()> {
    println!("{:?}", args);

    let generators = {
        let mut generators = Vec::new();
        for base in args.generators {
            let mut sub_files = find_files(&base, args.recursive).unwrap();
            generators.append(&mut sub_files);
        }
        generators
    };

    let langs: Vec<Box<dyn Language>> = if args.language.len() == 0 {
        vec![
            Box::new(Clang),
            Box::new(Cpp),
            Box::new(Python),
            Box::new(Txt),
        ]
    } else {
        let custom_lang =
            CustomLang::new(Regex::new(&args.language[0])?, args.language[1..].to_vec())?;
        vec![
            Box::new(custom_lang),
            Box::new(Clang),
            Box::new(Cpp),
            Box::new(Python),
            Box::new(Txt),
        ]
    };

    if !args.outdir.exists() {
        create_dir_all(&args.outdir)?;
    }

    let mut cases = Vec::new();
    for target in generators {
        if let Ok(mut sub_cases) =
            generate(&target, &args.outdir, args.count, args.seed, &langs)
        {
            println!("[GENERATED] {:?}", target);
            cases.append(&mut sub_cases);
        } else {
            println!("[IGNORED] {:?}", target);
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_genfileinfo() {
        let info = GenFileInfo::from(Path::new("dir/test.0.nocount.ext"));
        assert_eq!(info.name, String::from("test.0.nocount"));
        assert_eq!(info.count, None);
        assert_eq!(info.ext, String::from("ext"));

        let info = GenFileInfo::from(Path::new("test.5.ext"));
        assert_eq!(info.name, String::from("test"));
        assert_eq!(info.count, Some(5));
        assert_eq!(info.ext, String::from("ext"));
    }

    #[test]
    #[should_panic]
    fn test_genfileinfo_panic() {
        let _ = GenFileInfo::from(Path::new("0.ext"));
    }
}
