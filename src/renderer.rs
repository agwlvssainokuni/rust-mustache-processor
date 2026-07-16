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

//! ASTを`Value`コンテキストに対して評価し、出力文字列を生成するレンダラー（非公開）。

use crate::ast::{Node, SourcePosition};
use crate::error::{RenderError, RenderErrorKind};
use crate::partial::PartialResolver;
use crate::value::Value;

/// セクション・パーシャルの再帰ネストの最大深度（NFR Design Q2）。
pub(crate) const MAX_NESTING_DEPTH: usize = 100;

/// レンダリング全体で共有・更新される内部状態（NFR Design Q1）。
///
/// パーシャルの再帰は名前チェーン追跡による循環検出ではなく、`depth`の上限
/// （`MAX_NESTING_DEPTH`）のみで安全性を担保する（Step8で発見した設計補正、後述）。
pub(crate) struct RenderState<'a> {
    context_stack: Vec<&'a Value>,
    depth: usize,
    strict: bool,
}

impl<'a> RenderState<'a> {
    pub(crate) fn new(root: &'a Value, strict: bool) -> Self {
        Self {
            context_stack: vec![root],
            depth: 0,
            strict,
        }
    }
}

/// ASTノード列をレンダリングし、`out`に出力を追加する。
pub(crate) fn render_nodes(
    nodes: &[Node],
    state: &mut RenderState,
    partial_resolver: Option<&dyn PartialResolver>,
    out: &mut String,
) -> Result<(), RenderError> {
    for node in nodes {
        match node {
            Node::Text(text) => out.push_str(text),
            Node::Variable { name, escape, pos } => {
                render_variable(name, *escape, *pos, state, out)?;
            }
            Node::Section {
                name,
                inverted,
                children,
                pos,
            } => {
                render_section(name, *inverted, children, *pos, state, partial_resolver, out)?;
            }
            Node::Partial { name, indent, pos } => {
                render_partial(name, indent, *pos, state, partial_resolver, out)?;
            }
        }
    }
    Ok(())
}

/// 名前を解決して値を返す。
///
/// - `.`（暗黙のイテレータ）は、コンテキストスタックの最上位（現在のコンテキスト自体）を返す
/// - `.`を含む名前（例: `a.b.c`）はドット区切りのパスとして扱う。最初のセグメントのみ
///   コンテキストスタックを探索し（BR-4.1/BR-4.2）、以降のセグメントは直前で解決した値への
///   直接のキー参照として辿る（同名のフラットキー、例: データ中の`"a.b"`というキー自体は
///   絶対に参照しない）
/// - それ以外は単一キーとしてコンテキストスタックを探索する（BR-4.1/BR-4.2）
fn resolve<'a>(state: &RenderState<'a>, name: &str) -> Option<&'a Value> {
    if name == "." {
        return state.context_stack.last().copied();
    }

    let mut segments = name.split('.');
    let first = segments.next()?;
    let mut current = resolve_single(state, first)?;
    for segment in segments {
        current = current.get(segment)?;
    }
    Some(current)
}

/// コンテキストスタックを最も内側から探索し、単一キーに対応する値を返す（BR-4.1/BR-4.2）。
fn resolve_single<'a>(state: &RenderState<'a>, name: &str) -> Option<&'a Value> {
    for ctx in state.context_stack.iter().rev() {
        if let Some(v) = ctx.get(name) {
            return Some(v);
        }
    }
    None
}

fn render_variable(
    name: &str,
    escape: bool,
    pos: SourcePosition,
    state: &mut RenderState,
    out: &mut String,
) -> Result<(), RenderError> {
    match resolve(state, name) {
        None => {
            // BR-1.9: strictモードOFF（既定）なら空文字として継続。
            if state.strict {
                return Err(mk_render_error(
                    RenderErrorKind::UndefinedVariable {
                        name: name.to_string(),
                    },
                    pos,
                ));
            }
        }
        Some(v) => {
            let rendered = stringify(v);
            if escape {
                push_escaped(out, &rendered);
            } else {
                out.push_str(&rendered);
            }
        }
    }
    Ok(())
}

/// 値の文字列化（BR-1.3〜BR-1.8）。
fn stringify(value: &Value) -> String {
    match value {
        Value::Null => String::new(),
        Value::Bool(b) => {
            if *b {
                "true".to_string()
            } else {
                "false".to_string()
            }
        }
        Value::Integer(i) => i.to_string(),
        Value::Float(f) => stringify_float(*f),
        Value::String(s) => s.clone(),
        Value::Array(_) | Value::Map(_) => String::new(),
    }
}

fn stringify_float(f: f64) -> String {
    if f.is_nan() {
        return "NaN".to_string();
    }
    if f.is_infinite() {
        return if f > 0.0 {
            "inf".to_string()
        } else {
            "-inf".to_string()
        };
    }
    let s = f.to_string();
    if s.contains('.') || s.contains('e') || s.contains('E') {
        s
    } else {
        format!("{s}.0")
    }
}

/// HTMLエスケープ（BR-1.1）。
fn push_escaped(out: &mut String, s: &str) {
    for ch in s.chars() {
        match ch {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&#39;"),
            _ => out.push(ch),
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn render_section(
    name: &str,
    inverted: bool,
    children: &[Node],
    pos: SourcePosition,
    state: &mut RenderState,
    partial_resolver: Option<&dyn PartialResolver>,
    out: &mut String,
) -> Result<(), RenderError> {
    let value = resolve(state, name);
    // BR-2.5: 未定義キーは単に偽として扱う（strictモードでもエラーにならない）。
    let truthy = value.is_some_and(Value::is_truthy);
    let should_render = if inverted { !truthy } else { truthy };

    if !should_render {
        return Ok(());
    }

    match value {
        Some(Value::Array(items)) if !inverted => {
            // BR-2.2: 配列は各要素をコンテキストにプッシュして繰り返す。
            for item in items {
                enter_depth(state, pos)?;
                state.context_stack.push(item);
                let result = render_nodes(children, state, partial_resolver, out);
                state.context_stack.pop();
                state.depth -= 1;
                result?;
            }
        }
        Some(v) if !inverted => {
            // BR-2.3/BR-2.4: Map・スカラー真値のいずれも、その値自体を1回だけ
            // コンテキストにプッシュして表示する（公式spec準拠。`{{.}}`が
            // スカラーセクションの値自体を参照できるようにするため、Mapと
            // 同様にプッシュする必要がある。Step8のspec conformanceテストで発見）。
            enter_depth(state, pos)?;
            state.context_stack.push(v);
            let result = render_nodes(children, state, partial_resolver, out);
            state.context_stack.pop();
            state.depth -= 1;
            result?;
        }
        _ => {
            // 逆セクション（BR-3.1、valueが偽または未定義）はコンテキスト不変。
            enter_depth(state, pos)?;
            let result = render_nodes(children, state, partial_resolver, out);
            state.depth -= 1;
            result?;
        }
    }

    Ok(())
}

fn render_partial(
    name: &str,
    indent: &str,
    pos: SourcePosition,
    state: &mut RenderState,
    partial_resolver: Option<&dyn PartialResolver>,
    out: &mut String,
) -> Result<(), RenderError> {
    // BR-5.1/BR-5.2: 遅延評価で解決する。公式spec準拠でデフォルト（非strict）は
    // 空文字列として継続し、strictモードでは検出目的でエラーとする
    // （リゾルバ未設定・名前未解決のいずれも同様に扱う）。
    let content = match partial_resolver.and_then(|r| r.resolve(name)) {
        Some(c) => c,
        None => {
            if state.strict {
                return Err(mk_render_error(
                    RenderErrorKind::PartialNotFound {
                        name: name.to_string(),
                    },
                    pos,
                ));
            }
            return Ok(());
        }
    };

    // BR-5.4: インデントは値展開前のパーシャル・テンプレート文字列自体に適用する。
    // レンダリング後の出力に事後適用すると、展開された値そのものに含まれる改行にまで
    // インデントが波及してしまうため（Step8のspec conformanceテストで発見）。
    let indented_content = if indent.is_empty() {
        content
    } else {
        indent_source(&content, indent)
    };

    // BR-6.3: パーシャル内容はデフォルトデリミタから再パースする。
    let nodes = crate::parser::parse(&indented_content).map_err(|parse_err| RenderError {
        kind: RenderErrorKind::PartialParseError {
            name: name.to_string(),
            message: parse_err.message.clone(),
        },
        line: parse_err.line,
        column: parse_err.column,
        message: format!("failed to parse partial '{name}': {}", parse_err.message),
    })?;

    // 同名パーシャルの再帰は、公式spec上は正当な実装パターンとして許容される
    // （データに基づき自然終端するツリー/リスト構造の再帰的パーシャルなど）。
    // 名前チェーンによる循環検出は行わず、MAX_NESTING_DEPTHのみを安全装置とする。
    enter_depth(state, pos)?;
    let result = render_nodes(&nodes, state, partial_resolver, out);
    state.depth -= 1;
    result
}

/// パーシャルのテンプレート文字列に対し、各行（末尾改行のない最終行を除く）へ
/// `indent`を適用する（BR-5.4）。値展開前の静的テキストにのみ適用することで、
/// 展開された値の内容に含まれる改行にはインデントが波及しないようにする。
fn indent_source(content: &str, indent: &str) -> String {
    if content.is_empty() {
        return String::new();
    }
    let ends_with_newline = content.ends_with('\n');
    let body = if ends_with_newline {
        &content[..content.len() - 1]
    } else {
        content
    };
    let mut result = String::with_capacity(content.len() + indent.len());
    let mut first = true;
    for line in body.split('\n') {
        if !first {
            result.push('\n');
        }
        result.push_str(indent);
        result.push_str(line);
        first = false;
    }
    if ends_with_newline {
        result.push('\n');
    }
    result
}

fn enter_depth(state: &mut RenderState, pos: SourcePosition) -> Result<(), RenderError> {
    state.depth += 1;
    if state.depth > MAX_NESTING_DEPTH {
        return Err(mk_render_error(
            RenderErrorKind::MaxNestingDepthExceeded {
                depth: state.depth,
            },
            pos,
        ));
    }
    Ok(())
}

fn mk_render_error(kind: RenderErrorKind, pos: SourcePosition) -> RenderError {
    let message = describe_render_error(&kind);
    RenderError {
        kind,
        line: pos.line,
        column: pos.column,
        message,
    }
}

fn describe_render_error(kind: &RenderErrorKind) -> String {
    match kind {
        RenderErrorKind::UndefinedVariable { name } => format!("undefined variable: {name}"),
        RenderErrorKind::PartialNotFound { name } => format!("partial not found: {name}"),
        RenderErrorKind::MaxNestingDepthExceeded { depth } => {
            format!("maximum nesting depth ({depth}) exceeded")
        }
        RenderErrorKind::PartialParseError { name, message } => {
            format!("failed to parse partial '{name}': {message}")
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::value::Map;

    struct NoopResolver;
    impl PartialResolver for NoopResolver {
        fn resolve(&self, _name: &str) -> Option<String> {
            None
        }
    }

    struct MapResolver(std::collections::HashMap<&'static str, &'static str>);
    impl PartialResolver for MapResolver {
        fn resolve(&self, name: &str) -> Option<String> {
            self.0.get(name).map(|s| s.to_string())
        }
    }

    fn render(template: &str, data: &Value, strict: bool) -> Result<String, RenderError> {
        let nodes = crate::parser::parse(template).unwrap();
        let mut state = RenderState::new(data, strict);
        let mut out = String::new();
        render_nodes(&nodes, &mut state, None, &mut out)?;
        Ok(out)
    }

    fn render_with_resolver(
        template: &str,
        data: &Value,
        strict: bool,
        resolver: &dyn PartialResolver,
    ) -> Result<String, RenderError> {
        let nodes = crate::parser::parse(template).unwrap();
        let mut state = RenderState::new(data, strict);
        let mut out = String::new();
        render_nodes(&nodes, &mut state, Some(resolver), &mut out)?;
        Ok(out)
    }

    #[test]
    fn escapes_html_by_default() {
        let mut map = Map::new();
        map.insert("name", Value::String("<b>&'\"".to_string()));
        let out = render("{{name}}", &Value::Map(map), false).unwrap();
        assert_eq!(out, "&lt;b&gt;&amp;&#39;&quot;");
    }

    #[test]
    fn unescaped_variable_passes_through() {
        let mut map = Map::new();
        map.insert("name", Value::String("<b>".to_string()));
        let out = render("{{{name}}}", &Value::Map(map), false).unwrap();
        assert_eq!(out, "<b>");
    }

    #[test]
    fn undefined_variable_default_empty() {
        let out = render("[{{missing}}]", &Value::Map(Map::new()), false).unwrap();
        assert_eq!(out, "[]");
    }

    #[test]
    fn undefined_variable_strict_errors() {
        let err = render("[{{missing}}]", &Value::Map(Map::new()), true).unwrap_err();
        assert!(matches!(err.kind, RenderErrorKind::UndefinedVariable { .. }));
    }

    #[test]
    fn undefined_section_key_is_falsy_even_in_strict_mode() {
        // Q2=A: strictはvariable interpolationのみ対象。sectionの未定義キーはエラーにならない。
        let out = render("[{{#missing}}x{{/missing}}]", &Value::Map(Map::new()), true).unwrap();
        assert_eq!(out, "[]");
    }

    #[test]
    fn integer_and_float_rendering() {
        let mut map = Map::new();
        map.insert("i", Value::Integer(-3));
        map.insert("f", Value::Float(2.0));
        map.insert("g", Value::Float(1.5));
        let out = render("{{i}} {{f}} {{g}}", &Value::Map(map), false).unwrap();
        assert_eq!(out, "-3 2.0 1.5");
    }

    #[test]
    fn bool_and_null_rendering() {
        let mut map = Map::new();
        map.insert("t", Value::Bool(true));
        map.insert("f", Value::Bool(false));
        map.insert("n", Value::Null);
        let out = render("{{t}}/{{f}}/[{{n}}]", &Value::Map(map), false).unwrap();
        assert_eq!(out, "true/false/[]");
    }

    #[test]
    fn falsy_section_bool_false() {
        let mut map = Map::new();
        map.insert("a", Value::Bool(false));
        let out = render("[{{#a}}x{{/a}}]", &Value::Map(map), false).unwrap();
        assert_eq!(out, "[]");
    }

    #[test]
    fn falsy_section_empty_array() {
        let mut map = Map::new();
        map.insert("a", Value::Array(vec![]));
        let out = render("[{{#a}}x{{/a}}]", &Value::Map(map), false).unwrap();
        assert_eq!(out, "[]");
    }

    #[test]
    fn truthy_section_empty_string_and_empty_map() {
        // 実装時の追加補正: 公式spec準拠(business-rules.md BR-2.1〜2.4)により
        // 空文字列・空Mapはtruthyとして扱う。
        let mut map = Map::new();
        map.insert("s", Value::String(String::new()));
        map.insert("m", Value::Map(Map::new()));
        let out = render("[{{#s}}x{{/s}}][{{#m}}y{{/m}}]", &Value::Map(map), false).unwrap();
        assert_eq!(out, "[x][y]");
    }

    #[test]
    fn array_section_repeats_and_pushes_context() {
        let mut item1 = Map::new();
        item1.insert("n", Value::Integer(1));
        let mut item2 = Map::new();
        item2.insert("n", Value::Integer(2));
        let mut root = Map::new();
        root.insert(
            "items",
            Value::Array(vec![Value::Map(item1), Value::Map(item2)]),
        );
        let out = render("{{#items}}({{n}}){{/items}}", &Value::Map(root), false).unwrap();
        assert_eq!(out, "(1)(2)");
    }

    #[test]
    fn map_section_pushes_context_once() {
        let mut inner = Map::new();
        inner.insert("n", Value::Integer(42));
        let mut root = Map::new();
        root.insert("obj", Value::Map(inner));
        let out = render("{{#obj}}{{n}}{{/obj}}", &Value::Map(root), false).unwrap();
        assert_eq!(out, "42");
    }

    #[test]
    fn context_stack_inner_shadows_outer() {
        let mut inner = Map::new();
        inner.insert("n", Value::Integer(2));
        let mut root = Map::new();
        root.insert("n", Value::Integer(1));
        root.insert("obj", Value::Map(inner));
        let out = render("{{n}}{{#obj}}{{n}}{{/obj}}{{n}}", &Value::Map(root), false).unwrap();
        assert_eq!(out, "121");
    }

    #[test]
    fn inverted_section_renders_when_falsy() {
        let out = render("[{{^missing}}x{{/missing}}]", &Value::Map(Map::new()), false).unwrap();
        assert_eq!(out, "[x]");
    }

    #[test]
    fn inverted_section_skips_when_truthy() {
        let mut map = Map::new();
        map.insert("a", Value::Bool(true));
        let out = render("[{{^a}}x{{/a}}]", &Value::Map(map), false).unwrap();
        assert_eq!(out, "[]");
    }

    #[test]
    fn partial_without_resolver_renders_empty_by_default() {
        // 公式spec準拠（Failed Lookup）: リゾルバ未設定でもデフォルトは空文字列。
        let out = render("[{{> p}}]", &Value::Map(Map::new()), false).unwrap();
        assert_eq!(out, "[]");
    }

    #[test]
    fn partial_missing_renders_empty_by_default() {
        // 公式spec準拠（Failed Lookup）: 未解決のパーシャルはデフォルトで空文字列。
        let out = render_with_resolver(
            "[{{> p}}]",
            &Value::Map(Map::new()),
            false,
            &NoopResolver,
        )
        .unwrap();
        assert_eq!(out, "[]");
    }

    #[test]
    fn partial_missing_errors_in_strict_mode() {
        let err = render_with_resolver(
            "{{> p}}",
            &Value::Map(Map::new()),
            true,
            &NoopResolver,
        )
        .unwrap_err();
        assert!(matches!(err.kind, RenderErrorKind::PartialNotFound { .. }));
    }

    #[test]
    fn partial_resolves_and_renders_with_current_context() {
        let mut resolver = std::collections::HashMap::new();
        resolver.insert("p", "Hello, {{name}}!");
        let mut data = Map::new();
        data.insert("name", Value::String("World".to_string()));
        let out = render_with_resolver(
            "{{> p}}",
            &Value::Map(data),
            false,
            &MapResolver(resolver),
        )
        .unwrap();
        assert_eq!(out, "Hello, World!");
    }

    #[test]
    fn partial_self_recursion_terminates_via_data() {
        // 公式spec準拠（Recursion）: 同名パーシャルの再帰はデータに基づき正常終端する
        // 正当なパターンであり、循環としてエラーにしてはならない。
        let mut resolver = std::collections::HashMap::new();
        resolver.insert("node", "{{content}}<{{#nodes}}{{>node}}{{/nodes}}>");
        let mut leaf = Map::new();
        leaf.insert("content", Value::String("Y".to_string()));
        leaf.insert("nodes", Value::Array(vec![]));
        let mut root = Map::new();
        root.insert("content", Value::String("X".to_string()));
        root.insert("nodes", Value::Array(vec![Value::Map(leaf)]));

        let out = render_with_resolver(
            "{{>node}}",
            &Value::Map(root),
            false,
            &MapResolver(resolver),
        )
        .unwrap();
        assert_eq!(out, "X<Y<>>");
    }

    #[test]
    fn partial_infinite_recursion_hits_depth_guard() {
        // 真に無限のパーシャル再帰は、循環検出ではなくMAX_NESTING_DEPTHで停止する。
        let mut resolver = std::collections::HashMap::new();
        resolver.insert("a", "{{> a}}");
        let err = render_with_resolver(
            "{{> a}}",
            &Value::Map(Map::new()),
            false,
            &MapResolver(resolver),
        )
        .unwrap_err();
        assert!(matches!(
            err.kind,
            RenderErrorKind::MaxNestingDepthExceeded { .. }
        ));
    }

    #[test]
    fn partial_indent_applied_to_each_line() {
        let mut resolver = std::collections::HashMap::new();
        resolver.insert("p", "a\nb\n");
        let out = render_with_resolver(
            "  {{> p}}\n",
            &Value::Map(Map::new()),
            false,
            &MapResolver(resolver),
        )
        .unwrap();
        assert_eq!(out, "  a\n  b\n");
    }

    #[test]
    fn max_nesting_depth_exceeded() {
        // セクションが1000階層を超えてネストしていると深度超過エラーになる。
        let depth = MAX_NESTING_DEPTH + 1;
        let mut template = String::new();
        for _ in 0..depth {
            template.push_str("{{#a}}");
        }
        for _ in 0..depth {
            template.push_str("{{/a}}");
        }
        let mut inner = Value::Bool(true);
        for _ in 0..depth {
            let mut m = Map::new();
            m.insert("a", inner);
            inner = Value::Map(m);
        }
        let err = render(&template, &inner, false).unwrap_err();
        assert!(matches!(
            err.kind,
            RenderErrorKind::MaxNestingDepthExceeded { .. }
        ));
    }

    #[test]
    fn implicit_iterator_variable() {
        let out = render("Hello, {{.}}!", &Value::String("world".to_string()), false).unwrap();
        assert_eq!(out, "Hello, world!");
    }

    #[test]
    fn implicit_iterator_in_array_section() {
        let list = Value::Array(vec![
            Value::String("a".to_string()),
            Value::String("b".to_string()),
        ]);
        let mut root = Map::new();
        root.insert("list", list);
        let out = render("{{#list}}({{.}}){{/list}}", &Value::Map(root), false).unwrap();
        assert_eq!(out, "(a)(b)");
    }

    #[test]
    fn implicit_iterator_root_level_array() {
        let mut item1 = Map::new();
        item1.insert("value", Value::String("a".to_string()));
        let mut item2 = Map::new();
        item2.insert("value", Value::String("b".to_string()));
        let root = Value::Array(vec![Value::Map(item1), Value::Map(item2)]);
        let out = render("{{#.}}({{value}}){{/.}}", &root, false).unwrap();
        assert_eq!(out, "(a)(b)");
    }

    #[test]
    fn dotted_name_basic() {
        let mut person = Map::new();
        person.insert("name", Value::String("Joe".to_string()));
        let mut root = Map::new();
        root.insert("person", Value::Map(person));
        let out = render("{{person.name}}", &Value::Map(root), false).unwrap();
        assert_eq!(out, "Joe");
    }

    #[test]
    fn dotted_name_never_treated_as_flat_key() {
        let mut root = Map::new();
        root.insert("a.b", Value::String("c".to_string()));
        let out = render("[{{a.b}}]", &Value::Map(root), false).unwrap();
        assert_eq!(out, "[]");
    }

    #[test]
    fn dotted_name_no_masking_by_flat_key() {
        let mut nested = Map::new();
        nested.insert("b", Value::String("d".to_string()));
        let mut root = Map::new();
        root.insert("a.b", Value::String("c".to_string()));
        root.insert("a", Value::Map(nested));
        let out = render("{{a.b}}", &Value::Map(root), false).unwrap();
        assert_eq!(out, "d");
    }

    #[test]
    fn dotted_name_in_section() {
        let mut c = Map::new();
        c.insert("d", Value::Bool(true));
        let mut b = Map::new();
        b.insert("c", Value::Map(c));
        let mut a = Map::new();
        a.insert("b", Value::Map(b));
        let mut root = Map::new();
        root.insert("a", Value::Map(a));
        let out = render(
            "{{#a.b.c.d}}Here{{/a.b.c.d}}",
            &Value::Map(root),
            false,
        )
        .unwrap();
        assert_eq!(out, "Here");
    }

    #[test]
    fn partial_indent_not_applied_to_interpolated_value_newlines() {
        // 公式spec準拠（Standalone Indentation）: インデントは値展開前のパーシャル自身の
        // 行構造にのみ適用され、展開された値に含まれる改行には波及しない。
        let mut resolver = std::collections::HashMap::new();
        resolver.insert("partial", "|\n{{{content}}}\n|\n");
        let mut data = Map::new();
        data.insert("content", Value::String("<\n->".to_string()));
        let out = render_with_resolver(
            "\\\n {{>partial}}\n/\n",
            &Value::Map(data),
            false,
            &MapResolver(resolver),
        )
        .unwrap();
        assert_eq!(out, "\\\n |\n <\n->\n |\n/\n");
    }
}
