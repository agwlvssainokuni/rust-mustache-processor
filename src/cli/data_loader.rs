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
