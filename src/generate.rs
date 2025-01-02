use crate::language::{compile_and_get_runstep, ExecuteStatus, Language};
use crate::utils::{find_files, make_languages};
use anyhow::{ensure, Context, Result};
use clap::Args;
use log::{info, warn};
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

    /// timelimit for generating answer
    #[arg(visible_alias = "tl", long, default_value_t = 10.0)]
    timelimit: f64,

    /// COMMAND[0:-1] are the compile commands. COMMAND[-1] is execute command
    #[arg(
        short,
        long,
        value_name = "<EXT>,<COMMAND>,...",
        required = false,
        value_delimiter = ','
    )]
    language: Vec<String>,
}

#[derive(Debug)]
struct GenFileInfo {
    name: String,
    count: Option<u32>,
}

impl GenFileInfo {
    /// hoge.{count}.ext を解釈する
    /// count が u32 としてパースできる場合は name = "hoge"
    /// count が u32 としてパースできない場合は name = "hoge.{count}"
    fn new(path: &Path) -> Result<Self> {
        let stem = path.file_stem().unwrap().to_string_lossy().to_string();

        // count の解決
        let parts: Vec<&str> = stem.rsplitn(2, '.').collect();
        let count = parts.get(0).and_then(|s| s.parse::<u32>().ok());
        let name = if count.is_some() {
            parts
                .get(1)
                .map(|s| s.to_string())
                .with_context(|| format!("{path:?} is invalid form!"))?
        } else {
            stem
        };

        Ok(GenFileInfo { name, count })
    }
}

/// 生成されたテストケースへのパスを返す
fn generate(
    target: &Path,
    outdir: &Path,
    count: u32,
    seed: u32,
    timelimit: f64,
    langs: &Vec<Box<dyn Language>>,
) -> Result<Vec<(ExecuteStatus, PathBuf)>> {
    ensure!(target.exists(), "{target:?} not found!");

    let info = GenFileInfo::new(target)?;

    // compile
    let dir = TempDir::new()?;
    let runstep = compile_and_get_runstep(&dir, &target, &langs)?;

    // generate
    let count = info.count.unwrap_or(count);
    let mut generated_cases = Vec::new();
    for i in 0..count {
        let output_name = format!("{}_{:03}.in", &info.name, i);
        let output_path = outdir.join(output_name);
        let output = File::create(&output_path).unwrap();

        let status = runstep
            .execute(
                &dir,
                vec![(seed + i as u32).to_string()],
                Stdio::null(),
                output,
                Stdio::null(),
                Duration::from_secs_f64(timelimit),
            )
            .with_context(|| format!("failed to generate {:?} at seed = {:?}", target, seed + i))?;

        generated_cases.push((status, output_path.to_path_buf()));
    }

    Ok(generated_cases)
}

pub(super) fn root(args: GenerateArgs) -> Result<()> {
    info!("{:#?}", args);

    let generators = {
        let mut generators = Vec::new();
        for base in args.generators {
            let mut sub_files = find_files(&base, args.recursive).unwrap();
            generators.append(&mut sub_files);
        }
        generators
    };
    info!("generators = {generators:#?}");

    let langs = make_languages(&args.language)?;

    if !args.outdir.exists() {
        create_dir_all(&args.outdir)?;
    }

    for target in generators {
        match generate(
            &target,
            &args.outdir,
            args.count,
            args.seed,
            args.timelimit,
            &langs,
        ) {
            Ok(cases) => {
                for (status, case) in cases {
                    info!("[GENERATE] {case:?}, status = {status:?}");
                }
            }
            Err(err) => {
                warn!("[IGNORE] {:?}, reason = {:?}", target, err);
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_genfileinfo() {
        let info = GenFileInfo::new(Path::new("dir/test.0.nocount.ext")).unwrap();
        assert_eq!(info.name, String::from("test.0.nocount"));
        assert_eq!(info.count, None);

        let info = GenFileInfo::new(Path::new("test.5.ext")).unwrap();
        assert_eq!(info.name, String::from("test"));
        assert_eq!(info.count, Some(5));

        let info = GenFileInfo::new(Path::new("0.ext"));
        assert!(info.is_err());
    }
}
