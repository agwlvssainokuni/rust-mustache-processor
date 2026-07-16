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

//! パース時・レンダリング時に発生するエラー型。

use std::fmt;

/// テンプレートのパース時に発生するエラー。
#[derive(Debug, Clone, PartialEq)]
pub struct ParseError {
    /// エラーの種別。
    pub kind: ParseErrorKind,
    /// エラー発生箇所の行番号（1始まり）。
    pub line: usize,
    /// エラー発生箇所の列番号（1始まり）。
    pub column: usize,
    /// 人間可読なエラーメッセージ。
    pub message: String,
}

/// `ParseError`の種別。
#[derive(Debug, Clone, PartialEq)]
pub enum ParseErrorKind {
    /// テンプレートがタグの途中（未閉鎖のデリミタ）で終端した。
    UnexpectedEof,
    /// セクションの開始・終了タグが対応していない。
    UnbalancedSection {
        /// 対応が取れていないセクション名。
        name: String,
    },
    /// デリミタ変更タグ（`{{=...=}}`）の構文が不正。
    UnknownDelimiterSyntax,
    /// タグ名が空。
    EmptyTagName,
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "parse error at line {}, column {}: {}",
            self.line, self.column, self.message
        )
    }
}

impl std::error::Error for ParseError {}

/// テンプレートのレンダリング時に発生するエラー。
#[derive(Debug, Clone, PartialEq)]
pub struct RenderError {
    /// エラーの種別。
    pub kind: RenderErrorKind,
    /// エラー発生箇所の行番号（1始まり）。
    pub line: usize,
    /// エラー発生箇所の列番号（1始まり）。
    pub column: usize,
    /// 人間可読なエラーメッセージ。
    pub message: String,
}

/// `RenderError`の種別。
#[derive(Debug, Clone, PartialEq)]
pub enum RenderErrorKind {
    /// strictモードで未定義の変数を参照した（変数展開のみが対象）。
    UndefinedVariable {
        /// 未定義だった変数名。
        name: String,
    },
    /// strictモードでパーシャルの解決に失敗した（非strictモードでは空文字列として継続する。
    /// 公式spec準拠。同名パーシャルの自己再帰は正当なパターンとして許容されるため、循環検出
    /// ではなく`MaxNestingDepthExceeded`が安全装置として機能する）。
    PartialNotFound {
        /// 解決できなかったパーシャル名。
        name: String,
    },
    /// セクション・パーシャルの再帰ネストが最大深度を超過した。
    MaxNestingDepthExceeded {
        /// 超過時点でのネスト深度。
        depth: usize,
    },
    /// パーシャルとして読み込んだ内容の構文解析に失敗した。
    PartialParseError {
        /// 解析に失敗したパーシャル名。
        name: String,
        /// パース失敗の詳細メッセージ。
        message: String,
    },
}

impl fmt::Display for RenderError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "render error at line {}, column {}: {}",
            self.line, self.column, self.message
        )
    }
}

impl std::error::Error for RenderError {}

/// `Mustache::render_str`が返す統合エラー型。
#[derive(Debug, Clone, PartialEq)]
pub enum Error {
    /// パース時のエラー。
    Parse(ParseError),
    /// レンダリング時のエラー。
    Render(RenderError),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Parse(e) => write!(f, "{e}"),
            Error::Render(e) => write!(f, "{e}"),
        }
    }
}

impl std::error::Error for Error {}

impl From<ParseError> for Error {
    fn from(e: ParseError) -> Self {
        Error::Parse(e)
    }
}

impl From<RenderError> for Error {
    fn from(e: RenderError) -> Self {
        Error::Render(e)
    }
}
