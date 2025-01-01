use crate::language::{compile_and_get_runstep, default_languages, CommandStep, CustomLang};
use crate::utils::find_files;
use anyhow::{bail, ensure, Result};
use clap::Args;
use regex::Regex;
use std::fs::{create_dir_all, File};
use std::path::{Path, PathBuf};
use std::process::Stdio;
use std::time::Duration;
use tempfile::TempDir;

#[derive(Debug, Args)]
pub(super) struct SolveArgs {
    /// path to the solver
    #[arg(value_name = "SOLVER")]
    solver: PathBuf,

    /// directory containing the testcases or path to the testcase(*.in)
    #[arg(short, long, default_value = "./testcases/input")]
    testcases: Vec<PathBuf>,

    /// recursively search for testcases
    #[arg(short, long, default_value_t = false)]
    recursive: bool,

    ///
    #[arg(short, long, default_value = "./testcases/answer")]
    outdir: PathBuf,

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

/// answer 出力先を返す
fn solve<P: AsRef<Path>>(
    current_dir: P,
    target: &Path,
    outdir: &Path,
    run: &CommandStep,
    timelimit: f64,
) -> Result<PathBuf> {
    let input = File::open(&target)?;

    let name = target.file_stem().unwrap().to_string_lossy().to_string();
    let answer_path = outdir.join(format!("{name}.ans"));
    let answer = File::create(&answer_path)?;

    if let Ok(status) = run.execute(
        current_dir,
        Vec::new(),
        input,
        answer,
        Stdio::null(),
        Duration::from_secs_f64(timelimit),
    ) {
        ensure!(status.success(), "failed to run");

        Ok(answer_path.into())
    } else {
        bail!("failed to run")
    }
}

pub(super) fn root(args: SolveArgs) -> Result<()> {
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

    let dir = TempDir::new()?;
    let runstep = compile_and_get_runstep(&dir, &args.solver, &langs)?;
    for target in testcases {
        if let Ok(answer) = solve(&dir, &target, &args.outdir, &runstep, args.timelimit) {
            println!("[SOLVED] {:?}", answer);
        } else {
            println!("[FAILED] {:?}", target);
        }
    }

    Ok(())
}
