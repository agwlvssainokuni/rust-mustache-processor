// Copyright 2026 agwlvssainokuni
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! CLIのオーケストレーション（CliRunner）。

pub(crate) mod args;
pub(crate) mod data_loader;
pub(crate) mod io;

use std::fmt;
use std::process::ExitCode;

use args::CliArgsError;
use data_loader::DataLoaderError;
use io::IoError;
use mustache_processor::partial::DirectoryPartialResolver;
use mustache_processor::Mustache;

#[derive(Debug, Clone, PartialEq)]
pub(crate) enum CliError {
    Args(CliArgsError),
    Io(IoError),
    DataLoader(DataLoaderError),
    Mustache(mustache_processor::error::Error),
}

impl fmt::Display for CliError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CliError::Args(e) => write!(f, "{e}"),
            CliError::Io(e) => write!(f, "{e}"),
            CliError::DataLoader(e) => write!(f, "{e}"),
            CliError::Mustache(e) => write!(f, "{e}"),
        }
    }
}

impl std::error::Error for CliError {}

impl From<CliArgsError> for CliError {
    fn from(e: CliArgsError) -> Self {
        CliError::Args(e)
    }
}

impl From<IoError> for CliError {
    fn from(e: IoError) -> Self {
        CliError::Io(e)
    }
}

impl From<DataLoaderError> for CliError {
    fn from(e: DataLoaderError) -> Self {
        CliError::DataLoader(e)
    }
}

impl From<mustache_processor::error::Error> for CliError {
    fn from(e: mustache_processor::error::Error) -> Self {
        CliError::Mustache(e)
    }
}

/// 中核ロジック。テンプレートを指定順にパース・レンダリングし、結果を連結する
/// （BR-5.1〜BR-5.3, BR-6.1: process-then-cat、Atomic Output Buffering）。
fn run_inner(argv: &[String]) -> Result<(), CliError> {
    let args = args::parse_args(argv)?;

    let templates = io::read_templates(&args)?;
    let data_raw = io::read_data(&args)?;
    let format = data_loader::detect_format(args.format, args.data.as_deref())?;
    let data = data_loader::load(&data_raw, format)?;

    let mut out = String::new();
    for loaded in &templates {
        let partials_dir = io::resolve_partials_dir(&args, &loaded.source);
        let mustache = Mustache::new()
            .with_strict(args.strict)
            .with_partial_resolver(Box::new(DirectoryPartialResolver::new(partials_dir)));

        let rendered = mustache.render_str(&loaded.content, &data)?;
        out.push_str(&rendered);
    }

    io::write_output(&args, &out)?;
    Ok(())
}

/// `component-methods.md`が要求する公開シグネチャ。`run_inner`の薄いラッパー
/// （BR-7.1〜BR-7.3、NFR Design パターン1: Result-based Core Logic）。
pub(crate) fn run(argv: &[String]) -> ExitCode {
    match run_inner(argv) {
        Ok(()) => ExitCode::from(0),
        Err(e) => {
            eprintln!("mustache: {e}");
            ExitCode::from(1)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::PathBuf;

    fn temp_dir(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!(
            "mustache_processor_cli_mod_test_{}_{}",
            std::process::id(),
            name
        ));
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    fn argv(args: &[&str]) -> Vec<String> {
        std::iter::once("mustache".to_string())
            .chain(args.iter().map(|s| s.to_string()))
            .collect()
    }

    #[test]
    fn run_inner_renders_single_template_to_file() {
        let dir = temp_dir("single");
        fs::write(dir.join("t.tmpl"), "Hello, {{name}}!").unwrap();
        fs::write(dir.join("data.json"), r#"{"name": "World"}"#).unwrap();

        let out_path = dir.join("out.txt");
        let result = run_inner(&argv(&[
            dir.join("t.tmpl").to_str().unwrap(),
            "--data",
            dir.join("data.json").to_str().unwrap(),
            "--output",
            out_path.to_str().unwrap(),
        ]));

        assert!(result.is_ok(), "unexpected error: {result:?}");
        assert_eq!(fs::read_to_string(&out_path).unwrap(), "Hello, World!");

        fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn run_inner_concatenates_multiple_templates_in_order() {
        let dir = temp_dir("multi");
        fs::write(dir.join("a.tmpl"), "A={{a}}").unwrap();
        fs::write(dir.join("b.tmpl"), ";B={{b}}").unwrap();
        fs::write(dir.join("data.json"), r#"{"a": 1, "b": 2}"#).unwrap();

        let out_path = dir.join("out.txt");
        let result = run_inner(&argv(&[
            dir.join("a.tmpl").to_str().unwrap(),
            dir.join("b.tmpl").to_str().unwrap(),
            "--data",
            dir.join("data.json").to_str().unwrap(),
            "--output",
            out_path.to_str().unwrap(),
        ]));

        assert!(result.is_ok(), "unexpected error: {result:?}");
        assert_eq!(fs::read_to_string(&out_path).unwrap(), "A=1;B=2");

        fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn run_inner_uses_per_template_partials_dir() {
        let dir = temp_dir("partials");
        let sub_a = dir.join("a_dir");
        let sub_b = dir.join("b_dir");
        fs::create_dir_all(&sub_a).unwrap();
        fs::create_dir_all(&sub_b).unwrap();
        fs::write(sub_a.join("a.tmpl"), "{{> p}}").unwrap();
        fs::write(sub_a.join("p.mustache"), "from-a").unwrap();
        fs::write(sub_b.join("b.tmpl"), "{{> p}}").unwrap();
        fs::write(sub_b.join("p.mustache"), "from-b").unwrap();
        fs::write(dir.join("data.json"), "{}").unwrap();

        let out_path = dir.join("out.txt");
        let result = run_inner(&argv(&[
            sub_a.join("a.tmpl").to_str().unwrap(),
            sub_b.join("b.tmpl").to_str().unwrap(),
            "--data",
            dir.join("data.json").to_str().unwrap(),
            "--output",
            out_path.to_str().unwrap(),
        ]));

        assert!(result.is_ok(), "unexpected error: {result:?}");
        assert_eq!(fs::read_to_string(&out_path).unwrap(), "from-afrom-b");

        fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn run_inner_fails_atomically_when_one_template_errors() {
        let dir = temp_dir("atomic");
        fs::write(dir.join("a.tmpl"), "OK-A").unwrap();
        fs::write(dir.join("b.tmpl"), "{{#unbalanced").unwrap();
        fs::write(dir.join("data.json"), "{}").unwrap();

        let out_path = dir.join("out.txt");
        let result = run_inner(&argv(&[
            dir.join("a.tmpl").to_str().unwrap(),
            dir.join("b.tmpl").to_str().unwrap(),
            "--data",
            dir.join("data.json").to_str().unwrap(),
            "--output",
            out_path.to_str().unwrap(),
        ]));

        assert!(matches!(result, Err(CliError::Mustache(_))));
        assert!(!out_path.exists(), "output must not be written on failure");

        fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn run_inner_propagates_args_error() {
        let result = run_inner(&argv(&[]));
        assert!(matches!(result, Err(CliError::Args(_))));
    }

    #[test]
    fn run_returns_exit_code_zero_on_success() {
        let dir = temp_dir("exit_ok");
        fs::write(dir.join("t.tmpl"), "ok").unwrap();
        fs::write(dir.join("data.json"), "{}").unwrap();
        let out_path = dir.join("out.txt");

        let code = run(&argv(&[
            dir.join("t.tmpl").to_str().unwrap(),
            "--data",
            dir.join("data.json").to_str().unwrap(),
            "--output",
            out_path.to_str().unwrap(),
        ]));
        assert_eq!(code, ExitCode::from(0));

        fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn run_returns_exit_code_one_on_error() {
        let code = run(&argv(&[]));
        assert_eq!(code, ExitCode::from(1));
    }
}
