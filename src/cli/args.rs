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

//! コマンドライン引数の解析（CliArgs）。

use std::fmt;
use std::path::PathBuf;

use clap::Parser;

use super::data_loader::DataFormat;

/// 引数解析結果。
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct CliArgs {
    pub(crate) templates: Vec<PathBuf>,
    pub(crate) template_stdin: bool,
    pub(crate) data: Option<PathBuf>,
    pub(crate) output: Option<PathBuf>,
    pub(crate) partials_dir: Option<PathBuf>,
    pub(crate) strict: bool,
    pub(crate) format: Option<DataFormat>,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) enum CliArgsError {
    Clap(String),
    NoTemplateSpecified,
    TemplateAndStdinConflict,
    TemplateStdinAndDataStdinConflict,
    InvalidFormat { value: String },
}

impl fmt::Display for CliArgsError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CliArgsError::Clap(message) => write!(f, "{message}"),
            CliArgsError::NoTemplateSpecified => write!(
                f,
                "no template specified (provide a template file or use --template-stdin)"
            ),
            CliArgsError::TemplateAndStdinConflict => write!(
                f,
                "cannot specify both a template file and --template-stdin"
            ),
            CliArgsError::TemplateStdinAndDataStdinConflict => write!(
                f,
                "cannot read both template and data from stdin (specify --data <file>)"
            ),
            CliArgsError::InvalidFormat { value } => {
                write!(f, "invalid data format '{value}' (expected 'json' or 'yaml')")
            }
        }
    }
}

impl std::error::Error for CliArgsError {}

/// `clap`によるコマンドライン引数の生の解析結果。
#[derive(Parser, Debug)]
#[command(name = "mustache", version, about = "A Mustache template renderer")]
struct RawArgs {
    /// テンプレートファイル（複数指定可、指定順に処理・連結される）
    templates: Vec<PathBuf>,

    /// テンプレートを標準入力から読み込む（位置引数のテンプレートとは併用不可）
    #[arg(long)]
    template_stdin: bool,

    /// データファイル（未指定時は標準入力から読み込む）
    #[arg(long)]
    data: Option<PathBuf>,

    /// 出力先ファイル（未指定時は標準出力へ書き出す）
    #[arg(short, long)]
    output: Option<PathBuf>,

    /// パーシャル探索ディレクトリ（未指定時はテンプレートファイルごとのディレクトリ）
    #[arg(long)]
    partials_dir: Option<PathBuf>,

    /// 未定義変数の参照をエラーとするstrictモード
    #[arg(long)]
    strict: bool,

    /// データ形式を明示指定する（json または yaml）
    #[arg(long)]
    format: Option<String>,
}

/// コマンドライン引数を解析する（BR-1.1〜BR-1.8）。
pub(crate) fn parse_args(argv: &[String]) -> Result<CliArgs, CliArgsError> {
    let raw = RawArgs::try_parse_from(argv).map_err(|e| CliArgsError::Clap(e.to_string()))?;

    // BR-1.2: 位置引数と--template-stdinは同時指定不可。
    if raw.template_stdin && !raw.templates.is_empty() {
        return Err(CliArgsError::TemplateAndStdinConflict);
    }
    // BR-1.3: いずれも指定されない場合はエラー。
    if !raw.template_stdin && raw.templates.is_empty() {
        return Err(CliArgsError::NoTemplateSpecified);
    }
    // BR-1.5: --template-stdinかつ--data未指定は、テンプレート・データ双方が
    // 標準入力を要求するため矛盾する。
    if raw.template_stdin && raw.data.is_none() {
        return Err(CliArgsError::TemplateStdinAndDataStdinConflict);
    }

    let format = raw.format.as_deref().map(parse_format).transpose()?;

    Ok(CliArgs {
        templates: raw.templates,
        template_stdin: raw.template_stdin,
        data: raw.data,
        output: raw.output,
        partials_dir: raw.partials_dir,
        strict: raw.strict,
        format,
    })
}

/// `--format`の値を`DataFormat`へ変換する（大文字小文字を区別しない）。
fn parse_format(value: &str) -> Result<DataFormat, CliArgsError> {
    match value.to_ascii_lowercase().as_str() {
        "json" => Ok(DataFormat::Json),
        "yaml" | "yml" => Ok(DataFormat::Yaml),
        _ => Err(CliArgsError::InvalidFormat {
            value: value.to_string(),
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn argv(args: &[&str]) -> Vec<String> {
        std::iter::once("mustache".to_string())
            .chain(args.iter().map(|s| s.to_string()))
            .collect()
    }

    #[test]
    fn single_template_defaults() {
        let args = parse_args(&argv(&["template.tmpl"])).unwrap();
        assert_eq!(args.templates, vec![PathBuf::from("template.tmpl")]);
        assert!(!args.template_stdin);
        assert_eq!(args.data, None);
        assert_eq!(args.output, None);
        assert_eq!(args.partials_dir, None);
        assert!(!args.strict);
        assert_eq!(args.format, None);
    }

    #[test]
    fn multiple_templates_in_order() {
        let args = parse_args(&argv(&["a.tmpl", "b.tmpl", "c.tmpl"])).unwrap();
        assert_eq!(
            args.templates,
            vec![
                PathBuf::from("a.tmpl"),
                PathBuf::from("b.tmpl"),
                PathBuf::from("c.tmpl"),
            ]
        );
    }

    #[test]
    fn no_template_specified_errors() {
        let err = parse_args(&argv(&[])).unwrap_err();
        assert_eq!(err, CliArgsError::NoTemplateSpecified);
    }

    #[test]
    fn template_and_stdin_conflict_errors() {
        let err = parse_args(&argv(&["--template-stdin", "template.tmpl"])).unwrap_err();
        assert_eq!(err, CliArgsError::TemplateAndStdinConflict);
    }

    #[test]
    fn template_stdin_without_data_errors() {
        let err = parse_args(&argv(&["--template-stdin"])).unwrap_err();
        assert_eq!(err, CliArgsError::TemplateStdinAndDataStdinConflict);
    }

    #[test]
    fn template_stdin_with_data_ok() {
        let args = parse_args(&argv(&["--template-stdin", "--data", "data.json"])).unwrap();
        assert!(args.template_stdin);
        assert!(args.templates.is_empty());
        assert_eq!(args.data, Some(PathBuf::from("data.json")));
    }

    #[test]
    fn format_json_and_yaml_parsed_case_insensitively() {
        let args = parse_args(&argv(&["t.tmpl", "--format", "JSON"])).unwrap();
        assert_eq!(args.format, Some(DataFormat::Json));

        let args = parse_args(&argv(&["t.tmpl", "--format", "yaml"])).unwrap();
        assert_eq!(args.format, Some(DataFormat::Yaml));
    }

    #[test]
    fn invalid_format_errors() {
        let err = parse_args(&argv(&["t.tmpl", "--format", "toml"])).unwrap_err();
        assert_eq!(
            err,
            CliArgsError::InvalidFormat {
                value: "toml".to_string()
            }
        );
    }

    #[test]
    fn strict_and_output_and_partials_dir() {
        let args = parse_args(&argv(&[
            "t.tmpl",
            "--strict",
            "-o",
            "out.txt",
            "--partials-dir",
            "partials",
        ]))
        .unwrap();
        assert!(args.strict);
        assert_eq!(args.output, Some(PathBuf::from("out.txt")));
        assert_eq!(args.partials_dir, Some(PathBuf::from("partials")));
    }
}
