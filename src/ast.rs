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
        pos: SourcePosition,
    },
    Partial {
        name: String,
        indent: String,
        pos: SourcePosition,
    },
}
