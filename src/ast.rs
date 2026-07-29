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

//! パース済みテンプレートの内部AST表現（非公開）。

/// テンプレート内の位置情報（1始まりの行・列）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct SourcePosition {
    pub(crate) line: usize,
    pub(crate) column: usize,
}

/// パーシャル名の指定方法（BR-11.1、`~dynamic-names`対応）。
#[derive(Debug, Clone, PartialEq)]
pub(crate) enum PartialName {
    /// `{{> name}}`。テンプレート中に直接書かれた静的な名前。
    Static(String),
    /// `{{>* name}}`。`name`は変数名であり、レンダリング時にコンテキストから
    /// 解決した文字列をパーシャル名として用いる。
    Dynamic(String),
}

/// Parserが生成し、Rendererが消費する中間表現。
#[derive(Debug, Clone, PartialEq)]
pub(crate) enum Node {
    Text(String),
    Variable {
        name: String,
        escape: bool,
        pos: SourcePosition,
    },
    Section {
        name: String,
        inverted: bool,
        children: Vec<Node>,
        /// セクション本体（開始タグ直後〜終了タグ直前）の生テキスト。
        /// ラムダのセクション文脈呼び出し（BR-9.2）に使用する。
        raw: String,
        /// このセクションタグがパースされた時点で有効だったデリミタ（開始側）。
        /// ラムダのセクション文脈での返り値再パース（BR-9.3）に使用する。
        open: String,
        /// 同上（終了側）。
        close: String,
        pos: SourcePosition,
    },
    Partial {
        name: PartialName,
        indent: String,
        pos: SourcePosition,
    },
    /// `{{<parent}}...{{/parent}}`（テンプレート継承、`~inheritance`対応）。
    /// `name`は常に静的指定のみサポートする（BR-11.3）。
    Parent {
        name: String,
        /// 直下の`Node::Block`のみがオーバーライドとして意味を持つ（BR-10.2）。
        children: Vec<Node>,
        indent: String,
        pos: SourcePosition,
    },
    /// `{{$block}}...{{/block}}`（テンプレート継承のブロック）。
    /// `Node::Parent`の内側（オーバーライド定義）・外側（単独評価、BR-10.4）の両方で使う。
    Block {
        name: String,
        children: Vec<Node>,
        /// 開始タグ直後〜終了タグ直前の生テキスト。ブロック再インデント処理（BR-10.10）で
        /// 差し替え内容として使う場合に使用する。
        raw: String,
        /// 開始タグが行頭clearするか（BR-10.8）。
        open_clears_start: bool,
        /// 開始タグが行末clearするか（BR-10.8）。
        open_clears_end: bool,
        /// 終了タグが行頭clearするか（BR-10.8）。
        close_clears_start: bool,
        /// 終了タグが行末clearするか（BR-10.8）。
        close_clears_end: bool,
        /// 開始タグ直前の行頭空白（`open_clears_start`が真の場合のみ意味を持つ）。
        /// 展開箇所として使う場合、Rule4 Step2（BR-10.11）で使用する。
        open_indent: String,
        pos: SourcePosition,
    },
}
