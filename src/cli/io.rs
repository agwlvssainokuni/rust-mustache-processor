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

//! テンプレート・データの読み込みと出力の書き出し（IoController）。

use std::fmt;
use std::fs;
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};

use super::args::CliArgs;

/// テンプレートの出所。
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum TemplateSource {
    File(PathBuf),
    Stdin,
}

/// 読み込み済みのテンプレート。
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct LoadedTemplate {
    pub(crate) source: TemplateSource,
    pub(crate) content: String,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) enum IoError {
    TemplateRead { path: PathBuf, message: String },
    TemplateStdinRead { message: String },
    DataRead { message: String },
    OutputWrite { message: String },
}

impl fmt::Display for IoError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            IoError::TemplateRead { path, message } => {
                write!(f, "failed to read template '{}': {message}", path.display())
            }
            IoError::TemplateStdinRead { message } => {
                write!(f, "failed to read template from stdin: {message}")
            }
            IoError::DataRead { message } => write!(f, "failed to read data: {message}"),
            IoError::OutputWrite { message } => write!(f, "failed to write output: {message}"),
        }
    }
}

impl std::error::Error for IoError {}

/// テンプレートを指定順に読み込む（BR-2.1〜BR-2.3）。
pub(crate) fn read_templates(args: &CliArgs) -> Result<Vec<LoadedTemplate>, IoError> {
    if args.template_stdin {
        let mut content = String::new();
        io::stdin()
            .read_to_string(&mut content)
            .map_err(|e| IoError::TemplateStdinRead {
                message: e.to_string(),
            })?;
        return Ok(vec![LoadedTemplate {
            source: TemplateSource::Stdin,
            content,
        }]);
    }

    args.templates
        .iter()
        .map(|path| {
            let content = fs::read_to_string(path).map_err(|e| IoError::TemplateRead {
                path: path.clone(),
                message: e.to_string(),
            })?;
            Ok(LoadedTemplate {
                source: TemplateSource::File(path.clone()),
                content,
            })
        })
        .collect()
}

/// データを読み込む（`--data`指定時はファイル、未指定時は標準入力）。
pub(crate) fn read_data(args: &CliArgs) -> Result<String, IoError> {
    match &args.data {
        Some(path) => fs::read_to_string(path).map_err(|e| IoError::DataRead {
            message: format!("{}: {e}", path.display()),
        }),
        None => {
            let mut content = String::new();
            io::stdin()
                .read_to_string(&mut content)
                .map_err(|e| IoError::DataRead {
                    message: e.to_string(),
                })?;
            Ok(content)
        }
    }
}

/// パーシャルディレクトリを解決する（BR-4.1〜BR-4.3）。
pub(crate) fn resolve_partials_dir(args: &CliArgs, source: &TemplateSource) -> PathBuf {
    if let Some(dir) = &args.partials_dir {
        return dir.clone();
    }

    match source {
        TemplateSource::File(path) => path
            .parent()
            .map(Path::to_path_buf)
            .unwrap_or_else(|| PathBuf::from(".")),
        TemplateSource::Stdin => {
            std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."))
        }
    }
}

/// レンダリング結果を出力先へ書き出す（`--output`指定時はファイル、未指定時は標準出力）。
pub(crate) fn write_output(args: &CliArgs, content: &str) -> Result<(), IoError> {
    match &args.output {
        Some(path) => fs::write(path, content).map_err(|e| IoError::OutputWrite {
            message: format!("{}: {e}", path.display()),
        }),
        None => {
            let mut stdout = io::stdout();
            stdout
                .write_all(content.as_bytes())
                .map_err(|e| IoError::OutputWrite {
                    message: e.to_string(),
                })?;
            stdout.flush().map_err(|e| IoError::OutputWrite {
                message: e.to_string(),
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn base_args() -> CliArgs {
        CliArgs {
            templates: vec![],
            template_stdin: false,
            data: None,
            output: None,
            partials_dir: None,
            strict: false,
            format: None,
        }
    }

    fn temp_dir(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!(
            "mustache_processor_cli_test_{}_{}",
            std::process::id(),
            name
        ));
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn read_templates_reads_multiple_files_in_order() {
        let dir = temp_dir("read_templates_multi");
        fs::write(dir.join("a.tmpl"), "A").unwrap();
        fs::write(dir.join("b.tmpl"), "B").unwrap();

        let mut args = base_args();
        args.templates = vec![dir.join("a.tmpl"), dir.join("b.tmpl")];

        let loaded = read_templates(&args).unwrap();
        assert_eq!(loaded.len(), 2);
        assert_eq!(loaded[0].content, "A");
        assert_eq!(loaded[1].content, "B");
        assert_eq!(loaded[0].source, TemplateSource::File(dir.join("a.tmpl")));

        fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn read_templates_missing_file_errors() {
        let dir = temp_dir("read_templates_missing");
        let mut args = base_args();
        args.templates = vec![dir.join("missing.tmpl")];

        let err = read_templates(&args).unwrap_err();
        assert!(matches!(err, IoError::TemplateRead { .. }));

        fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn read_data_from_file() {
        let dir = temp_dir("read_data_file");
        fs::write(dir.join("data.json"), "{}").unwrap();

        let mut args = base_args();
        args.data = Some(dir.join("data.json"));

        assert_eq!(read_data(&args).unwrap(), "{}");

        fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn resolve_partials_dir_explicit_overrides_all() {
        let mut args = base_args();
        args.partials_dir = Some(PathBuf::from("/explicit"));

        let dir = resolve_partials_dir(&args, &TemplateSource::File(PathBuf::from("/a/b.tmpl")));
        assert_eq!(dir, PathBuf::from("/explicit"));

        let dir = resolve_partials_dir(&args, &TemplateSource::Stdin);
        assert_eq!(dir, PathBuf::from("/explicit"));
    }

    #[test]
    fn resolve_partials_dir_defaults_to_template_file_directory() {
        let args = base_args();
        let dir = resolve_partials_dir(
            &args,
            &TemplateSource::File(PathBuf::from("/a/b/template.tmpl")),
        );
        assert_eq!(dir, PathBuf::from("/a/b"));
    }

    #[test]
    fn resolve_partials_dir_defaults_to_cwd_for_stdin() {
        let args = base_args();
        let dir = resolve_partials_dir(&args, &TemplateSource::Stdin);
        assert_eq!(dir, std::env::current_dir().unwrap());
    }

    #[test]
    fn write_output_to_file() {
        let dir = temp_dir("write_output_file");
        let mut args = base_args();
        args.output = Some(dir.join("out.txt"));

        write_output(&args, "hello").unwrap();
        assert_eq!(fs::read_to_string(dir.join("out.txt")).unwrap(), "hello");

        fs::remove_dir_all(&dir).ok();
    }
}
