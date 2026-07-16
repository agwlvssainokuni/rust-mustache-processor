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

//! JSON/YAMLデータのパースとcore-engineの`Value`への変換（DataLoader）。

use std::fmt;
use std::path::Path;

use mustache_processor::value::Value;

/// データの入力形式。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum DataFormat {
    Json,
    Yaml,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) enum DataLoaderError {
    UnknownFormat,
    Parse { format: DataFormat, message: String },
}

impl fmt::Display for DataLoaderError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DataLoaderError::UnknownFormat => write!(
                f,
                "cannot determine data format (specify --format or use a .json/.yaml/.yml data file)"
            ),
            DataLoaderError::Parse { format, message } => {
                let name = match format {
                    DataFormat::Json => "JSON",
                    DataFormat::Yaml => "YAML",
                };
                write!(f, "failed to parse {name} data: {message}")
            }
        }
    }
}

impl std::error::Error for DataLoaderError {}

/// データ形式を判定する（BR-3.1〜BR-3.3）: `--format`最優先、次にデータファイルの拡張子。
///
/// `component-methods.md`の概要シグネチャ`detect_format(args: &CliArgs) -> ...`は、
/// `CliArgs`が本モジュールの`DataFormat`型に依存するため、そのまま`&CliArgs`を
/// 引数に取ると`args`⇄`data_loader`の循環モジュール依存が生じる。判定に必要な
/// フィールドのみを引数に取る形に詳細化した（Code Generation時の補正）。
pub(crate) fn detect_format(
    explicit_format: Option<DataFormat>,
    data_path: Option<&Path>,
) -> Result<DataFormat, DataLoaderError> {
    if let Some(format) = explicit_format {
        return Ok(format);
    }

    let extension = data_path
        .and_then(|path| path.extension())
        .and_then(|ext| ext.to_str())
        .map(str::to_ascii_lowercase);

    match extension.as_deref() {
        Some("json") => Ok(DataFormat::Json),
        Some("yaml") | Some("yml") => Ok(DataFormat::Yaml),
        _ => Err(DataLoaderError::UnknownFormat),
    }
}

/// データ文字列を指定形式でパースし、core-engineの`Value`へ変換する（BR-3.4）。
pub(crate) fn load(raw: &str, format: DataFormat) -> Result<Value, DataLoaderError> {
    match format {
        DataFormat::Json => {
            let json: serde_json::Value =
                serde_json::from_str(raw).map_err(|e| DataLoaderError::Parse {
                    format,
                    message: e.to_string(),
                })?;
            Value::from_serialize(&json).map_err(|e| DataLoaderError::Parse {
                format,
                message: e.to_string(),
            })
        }
        DataFormat::Yaml => {
            let yaml: serde_norway::Value =
                serde_norway::from_str(raw).map_err(|e| DataLoaderError::Parse {
                    format,
                    message: e.to_string(),
                })?;
            Value::from_serialize(&yaml).map_err(|e| DataLoaderError::Parse {
                format,
                message: e.to_string(),
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn detect_format_prefers_explicit_over_extension() {
        let result = detect_format(Some(DataFormat::Yaml), Some(&PathBuf::from("data.json")));
        assert_eq!(result, Ok(DataFormat::Yaml));
    }

    #[test]
    fn detect_format_falls_back_to_extension() {
        assert_eq!(
            detect_format(None, Some(&PathBuf::from("data.json"))),
            Ok(DataFormat::Json)
        );
        assert_eq!(
            detect_format(None, Some(&PathBuf::from("data.yaml"))),
            Ok(DataFormat::Yaml)
        );
        assert_eq!(
            detect_format(None, Some(&PathBuf::from("data.yml"))),
            Ok(DataFormat::Yaml)
        );
    }

    #[test]
    fn detect_format_unknown_without_format_or_extension() {
        assert_eq!(
            detect_format(None, None),
            Err(DataLoaderError::UnknownFormat)
        );
        assert_eq!(
            detect_format(None, Some(&PathBuf::from("data.txt"))),
            Err(DataLoaderError::UnknownFormat)
        );
    }

    #[test]
    fn load_json_object() {
        let value = load(r#"{"name": "World", "n": 1}"#, DataFormat::Json).unwrap();
        match value {
            Value::Map(map) => {
                assert_eq!(map.get("name"), Some(&Value::String("World".to_string())));
                assert_eq!(map.get("n"), Some(&Value::Integer(1)));
            }
            other => panic!("expected Map, got {other:?}"),
        }
    }

    #[test]
    fn load_yaml_object() {
        let value = load("name: World\nn: 1\n", DataFormat::Yaml).unwrap();
        match value {
            Value::Map(map) => {
                assert_eq!(map.get("name"), Some(&Value::String("World".to_string())));
                assert_eq!(map.get("n"), Some(&Value::Integer(1)));
            }
            other => panic!("expected Map, got {other:?}"),
        }
    }

    #[test]
    fn load_invalid_json_errors() {
        let err = load("{not valid", DataFormat::Json).unwrap_err();
        assert!(matches!(
            err,
            DataLoaderError::Parse {
                format: DataFormat::Json,
                ..
            }
        ));
    }

    #[test]
    fn load_invalid_yaml_errors() {
        let err = load("- a\n  b: [unterminated", DataFormat::Yaml).unwrap_err();
        assert!(matches!(
            err,
            DataLoaderError::Parse {
                format: DataFormat::Yaml,
                ..
            }
        ));
    }
}
