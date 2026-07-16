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

//! Rustで実装したMustacheテンプレートエンジン。
//!
//! # 使用例
//!
//! ```
//! use mustache_processor::Mustache;
//! use mustache_processor::value::{Map, Value};
//!
//! let mustache = Mustache::new();
//! let mut data = Map::new();
//! data.insert("name", Value::String("World".to_string()));
//! let output = mustache
//!     .render_str("Hello, {{name}}!", &Value::Map(data))
//!     .unwrap();
//! assert_eq!(output, "Hello, World!");
//! ```

#![deny(missing_docs)]

mod ast;
mod parser;
mod renderer;

pub mod error;
pub mod partial;
pub mod value;

use crate::ast::Node;
use crate::error::{Error, ParseError, RenderError};
use crate::partial::PartialResolver;
use crate::renderer::RenderState;
use crate::value::Value;

/// パース済みテンプレート。
///
/// パース結果を保持する不透明な値。内部構造は公開せず、`Mustache::render`にのみ渡せる。
pub struct Template {
    pub(crate) root: Vec<Node>,
    pub(crate) source_len: usize,
}

/// Mustacheテンプレートエンジン。
///
/// パーシャルリゾルバやstrictモード等の設定を保持し、パース・レンダリングの起点となる。
pub struct Mustache {
    partial_resolver: Option<Box<dyn PartialResolver>>,
    strict: bool,
}

impl Default for Mustache {
    fn default() -> Self {
        Self::new()
    }
}

impl Mustache {
    /// デフォルト設定（パーシャルリゾルバなし、strict=false）でエンジンを作成する。
    pub fn new() -> Self {
        Self {
            partial_resolver: None,
            strict: false,
        }
    }

    /// パーシャルリゾルバを設定する（ビルダースタイル）。
    pub fn with_partial_resolver(mut self, resolver: Box<dyn PartialResolver>) -> Self {
        self.partial_resolver = Some(resolver);
        self
    }

    /// 未定義変数をエラーにするstrictモードを設定する（FR-7）。
    pub fn with_strict(mut self, strict: bool) -> Self {
        self.strict = strict;
        self
    }

    /// テンプレート文字列をパースし、再利用可能な`Template`を得る。
    pub fn parse(&self, template: &str) -> Result<Template, ParseError> {
        let root = parser::parse(template)?;
        Ok(Template {
            root,
            source_len: template.len(),
        })
    }

    /// パース済みテンプレートをデータでレンダリングする。
    pub fn render(&self, template: &Template, data: &Value) -> Result<String, RenderError> {
        let mut out = String::with_capacity(template.source_len);
        let mut state = RenderState::new(data, self.strict);
        renderer::render_nodes(
            &template.root,
            &mut state,
            self.partial_resolver.as_deref(),
            &mut out,
        )?;
        Ok(out)
    }

    /// パース＋レンダリングを1回で行う簡易メソッド。
    pub fn render_str(&self, template: &str, data: &Value) -> Result<String, Error> {
        let parsed = self.parse(template)?;
        let output = self.render(&parsed, data)?;
        Ok(output)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::partial::DirectoryPartialResolver;
    use crate::value::Map;

    #[test]
    fn render_str_parses_and_renders() {
        let mustache = Mustache::new();
        let mut data = Map::new();
        data.insert("name", Value::String("World".to_string()));
        let out = mustache
            .render_str("Hello, {{name}}!", &Value::Map(data))
            .unwrap();
        assert_eq!(out, "Hello, World!");
    }

    #[test]
    fn parse_then_render_reuses_template() {
        let mustache = Mustache::new();
        let template = mustache.parse("{{a}}-{{b}}").unwrap();

        let mut data1 = Map::new();
        data1.insert("a", Value::Integer(1));
        data1.insert("b", Value::Integer(2));
        assert_eq!(
            mustache.render(&template, &Value::Map(data1)).unwrap(),
            "1-2"
        );

        let mut data2 = Map::new();
        data2.insert("a", Value::Integer(9));
        data2.insert("b", Value::Integer(8));
        assert_eq!(
            mustache.render(&template, &Value::Map(data2)).unwrap(),
            "9-8"
        );
    }

    #[test]
    fn render_str_propagates_parse_error() {
        let mustache = Mustache::new();
        let err = mustache
            .render_str("{{#a}}unclosed", &Value::Map(Map::new()))
            .unwrap_err();
        assert!(matches!(err, Error::Parse(_)));
    }

    #[test]
    fn render_str_propagates_render_error() {
        let mustache = Mustache::new().with_strict(true);
        let err = mustache
            .render_str("{{missing}}", &Value::Map(Map::new()))
            .unwrap_err();
        assert!(matches!(err, Error::Render(_)));
    }

    #[test]
    fn with_partial_resolver_is_used() {
        let dir = std::env::temp_dir().join(format!(
            "mustache_processor_test_{}_lib_partial",
            std::process::id()
        ));
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("greeting.mustache"), "Hi, {{name}}!").unwrap();

        let mustache =
            Mustache::new().with_partial_resolver(Box::new(DirectoryPartialResolver::new(&dir)));
        let mut data = Map::new();
        data.insert("name", Value::String("Rust".to_string()));
        let out = mustache
            .render_str("{{> greeting}}", &Value::Map(data))
            .unwrap();
        assert_eq!(out, "Hi, Rust!");

        std::fs::remove_dir_all(&dir).ok();
    }
}
