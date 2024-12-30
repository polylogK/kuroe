use crate::language::{default_languages, detect_language, CommandStep, CustomLang};
use crate::utils::find_files;
use anyhow::{bail, Result};
use regex::Regex;
use std::fs::{create_dir_all, File};
use std::path::{Path, PathBuf};
use std::process::Stdio;
use std::time::Duration;
use tempfile::TempDir;

use clap::Args;

#[derive(Debug, Args)]
pub(super) struct ValidateArgs {
    /// directory containing the testcases or path to the testcase(*.in)
    #[arg(value_name = "TARGET", required = true)]
    testcases: Vec<PathBuf>,

    /// path to the validator
    #[arg(short, long, value_name = "VALIDATOR", required = true)]
    validator: PathBuf,

    /// recursively search for testcases
    #[arg(short, long, default_value_t = false)]
    recursive: bool,

    ///
    #[arg(short, long, default_value = "./testcases/validate")]
    outdir: PathBuf,

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

/// vaildate の結果とエラー出力先パスを返す
fn validate<P: AsRef<Path>>(
    current_dir: P,
    target: &Path,
    outdir: &Path,
    run: &CommandStep,
) -> Result<(bool, PathBuf)> {
    let input = File::open(&target)?;

    let name = target.file_stem().unwrap().to_string_lossy().to_string();
    let err_path = outdir.join(format!("{name}.val"));
    let err = File::create(&err_path)?;

    if let Ok(status) = run.execute(
        current_dir,
        Vec::new(),
        input,
        Stdio::null(),
        err,
        Duration::from_secs(10),
    ) {
        Ok((status.success(), err_path.into()))
    } else {
        bail!("failed to run")
    }
}

pub(super) fn root(args: ValidateArgs) -> Result<()> {
    println!("{:?}", args);

    let testcases = {
        let mut testcases = Vec::new();
        for base in args.testcases {
            let sub_files = find_files(&base, args.recursive).unwrap();

            for target in sub_files {
                if let Some(ext) = target.extension() {
                    if ext == "in" {
                        testcases.push(target);
                    }
                }
            }
        }
        testcases
    };

    let langs = if args.language.len() == 0 {
        default_languages()
    } else {
        let mut langs = default_languages();
        let custom_lang =
            CustomLang::new(Regex::new(&args.language[0])?, args.language[1..].to_vec())?;
        langs.insert(0, Box::new(custom_lang));
        langs
    };

    if !args.outdir.exists() {
        create_dir_all(&args.outdir)?;
    }

    // validator コンパイル
    let dir = TempDir::new()?;
    let runstep = {
        let lang = {
            let ext = args
                .validator
                .extension()
                .unwrap()
                .to_string_lossy()
                .to_string();
            detect_language(&ext, &langs)?
        };

        for step in lang.compile(&args.validator)? {
            step.execute(
                &dir,
                Vec::new(),
                Stdio::null(),
                Stdio::null(),
                Stdio::null(),
                Duration::from_secs(10),
            )?;
        }

        lang.run(&args.validator)?
    };

    for target in testcases {
        if let Ok((status, _)) = validate(&dir, &target, &args.outdir, &runstep) {
            println!("[VALIDATED] {:?}, status = {:?}", target, status);
        } else {
            println!("[IGNORED] {:?}", target);
        }
    }

    Ok(())
}
