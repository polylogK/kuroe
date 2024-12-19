use anyhow::{bail, Context, Result};
use std::path::Path;
use std::process::{Command, Stdio};
use std::time::Duration;
use wait_timeout::ChildExt;

#[derive(Debug)]
pub(crate) struct CommandStep {
    program: String,
    args: Vec<String>,
}

impl CommandStep {
    pub(crate) fn new(program: String, args: Vec<String>) -> Self {
        Self { program, args }
    }

    pub(crate) fn execute<P: AsRef<Path>, T: Into<Stdio>, U: Into<Stdio>>(
        &self,
        current_dir: P,
        stdin: T,
        stdout: U,
        time_limit: Duration,
    ) -> Result<()> {
        let mut child = Command::new(&self.program)
            .args(&self.args)
            .current_dir(current_dir)
            .stdin(stdin)
            .stdout(stdout)
            .spawn()
            .with_context(|| format!("Failed to execute {:?}", self))?;

        let status = match child.wait_timeout(time_limit)? {
            Some(status) => status,
            None => {
                // child hasn't exited yet
                child.kill().unwrap();
                child.wait().unwrap()
            }
        };
        if !status.success() {
            bail!("Failed to execute {:?} with status {:?}", self, status);
        }
        Ok(())
    }
}

pub(crate) trait Language {
    fn is_valid_ext(&self, ext: &str) -> bool;
    fn compile(&self, target: &Path) -> Vec<CommandStep>;
    fn run(&self, target: &Path, seed: i32) -> CommandStep;
}

pub(crate) struct Cpp;
impl Language for Cpp {
    fn is_valid_ext(&self, ext: &str) -> bool {
        return ext == "cpp" || ext == "cc";
    }

    fn compile(&self, target: &Path) -> Vec<CommandStep> {
        vec![CommandStep::new(
            "g++".to_string(),
            vec![
                "-std=c++20".to_string(),
                "-O2".to_string(),
                target.to_string_lossy().to_string(),
            ],
        )]
    }

    fn run(&self, _target: &Path, seed: i32) -> CommandStep {
        CommandStep::new("./a.out".to_string(), vec![seed.to_string()])
    }
}

pub(crate) struct Txt;
impl Language for Txt {
    fn is_valid_ext(&self, ext: &str) -> bool {
        return ext == "txt" || ext == "in";
    }

    fn compile(&self, _target: &Path) -> Vec<CommandStep> {
        Vec::new()
    }

    fn run(&self, target: &Path, _seed: i32) -> CommandStep {
        CommandStep::new(
            "cat".to_string(),
            vec![target.to_string_lossy().to_string()],
        )
    }
}

/// langs から ext に合った Language をクローンして返す
/// 複数の言語に合致する場合は先頭に近いものが優先される
/// カスタム言語で ext が被った場合はカスタム言語を先頭にすることで上書きすることが可能
pub(crate) fn detect_language<'a>(
    ext: &str,
    langs: &'a Vec<Box<dyn Language>>,
) -> Result<&'a Box<dyn Language>> {
    for lang in langs {
        if lang.is_valid_ext(ext) {
            return Ok(lang);
        }
    }
    bail!("no language detected");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_execute() {
        let step = CommandStep::new("true".to_string(), Vec::new());
        assert!(step
            .execute("./", Stdio::null(), Stdio::null(), Duration::from_secs(1))
            .is_ok());

        let step = CommandStep::new("false".to_string(), Vec::new());
        assert!(step
            .execute("./", Stdio::null(), Stdio::null(), Duration::from_secs(1))
            .is_err());
    }

    #[test]
    fn test_cpp() {
        let cpp = Cpp;
        assert_eq!(cpp.is_valid_ext("cpp"), true);
        assert_eq!(cpp.is_valid_ext("cc"), true);
        assert_eq!(cpp.is_valid_ext("test"), false);

        let cmd = cpp.run(Path::new(""), 0);
        assert_eq!(cmd.program, "./a.out".to_string());
        assert_eq!(cmd.args.len(), 1);
    }

    #[test]
    fn test_detect_language() {
        let langs: Vec<Box<dyn Language>> = vec![Box::new(Cpp), Box::new(Txt)];

        let lang = detect_language("cpp", &langs);
        assert!(lang.unwrap().is_valid_ext("cpp"));

        let lang = detect_language("txt", &langs);
        assert!(lang.unwrap().is_valid_ext("txt"));

        let lang = detect_language("test", &langs);
        assert!(lang.is_err());
    }

    use std::fs::{read_to_string, File};
    use tempfile::tempdir;

    #[test]
    fn test_compile_and_run() {
        let dir = tempdir().unwrap();

        // hello プログラムの作成
        let hello_path = dir.path().join("hello.cpp");
        let hello = File::create(&hello_path).unwrap();
        CommandStep::new(
            "echo".to_string(),
            vec!["#include <cstdio>\nint main(){ printf(\"hello\"); }".to_string()],
        )
        .execute(&dir, Stdio::null(), hello, Duration::from_secs(2))
        .unwrap();

        // コンパイル
        let cpp = Cpp;
        for step in cpp.compile(&hello_path) {
            step.execute(&dir, Stdio::null(), Stdio::null(), Duration::from_secs(2))
                .unwrap();
        }

        // 実行
        let output_path = dir.path().join("output.txt");
        let output = File::create(&output_path).unwrap();
        cpp.run(Path::new(""), 0)
            .execute(&dir, Stdio::null(), output, Duration::from_secs(2))
            .unwrap();

        assert_eq!(read_to_string(&output_path).unwrap(), "hello");
    }
}
