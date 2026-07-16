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

//! Mustacheテンプレート文字列を構文解析し、ASTを生成するパーサー（非公開）。
//!
//! 3パス構成:
//! 1. `tokenize`: デリミタ変更を追跡しつつ、テキストとタグをフラットな`Atom`列に分解する
//! 2. `apply_standalone_trimming`: 行単位でスタンドアロンタグ（BR-7.1/BR-7.2）を判定し、
//!    該当する行の前後の空白・改行を除去する（1行に複数のブロックタグが並ぶ場合や
//!    `\r\n`改行にも対応するため、単純な前後トークンの局所判定ではなく行単位で判定する）
//! 3. `build_tree`: クリーニング済みの`Atom`列からセクションの入れ子構造を持つASTを構築する

use crate::ast::{Node, SourcePosition};
use crate::error::{ParseError, ParseErrorKind};

/// テンプレート文字列からASTノード列を生成する。
pub(crate) fn parse(template: &str) -> Result<Vec<Node>, ParseError> {
    let atoms = tokenize(template)?;
    let atoms = apply_standalone_trimming(atoms);
    build_tree(atoms)
}

enum ParsedTag {
    Variable { name: String, escape: bool },
    SectionStart { name: String },
    Inverted { name: String },
    SectionEnd { name: String },
    Partial { name: String },
    Comment,
    DelimChange { open: String, close: String },
}

impl ParsedTag {
    fn is_block_type(&self) -> bool {
        !matches!(self, ParsedTag::Variable { .. })
    }
}

enum Atom {
    /// 改行を含まないテキスト断片。`newline`は直後にあった改行の生バイト列
    /// （`"\n"`または`"\r\n"`）。行末に改行がない場合（EOF）は`None`。
    Text {
        content: String,
        newline: Option<&'static str>,
    },
    Tag {
        parsed: ParsedTag,
        pos: SourcePosition,
    },
}

/// Pass 2以降で扱う、スタンドアロン判定用にインデント情報を持てるように拡張した要素。
enum Item {
    Text {
        content: String,
        newline: Option<&'static str>,
    },
    Tag {
        parsed: ParsedTag,
        pos: SourcePosition,
        indent: String,
    },
}

/// Pass 1: デリミタ変更を追跡しながら、テキストとタグをフラットな`Atom`列に分解する。
/// スタンドアロン判定はここでは行わない。
fn tokenize(template: &str) -> Result<Vec<Atom>, ParseError> {
    let mut scanner = Scanner::new(template);
    let mut open = String::from("{{");
    let mut close = String::from("}}");
    let mut atoms: Vec<Atom> = Vec::new();

    loop {
        let search_start = scanner.pos;
        let tag_start = match find_from(scanner.src, search_start, &open) {
            Some(p) => p,
            None => {
                push_text_atoms(&mut atoms, &scanner.src[search_start..]);
                break;
            }
        };

        let text_before = &scanner.src[search_start..tag_start];
        push_text_atoms(&mut atoms, text_before);

        let tag_pos = scanner.position_at(tag_start);
        let (tag_end, parsed) = scan_tag(scanner.src, tag_start, &open, &close)
            .map_err(|kind| to_parse_error(kind, tag_pos))?;

        if let ParsedTag::DelimChange {
            open: new_open,
            close: new_close,
        } = &parsed
        {
            open = new_open.clone();
            close = new_close.clone();
        }

        atoms.push(Atom::Tag {
            parsed,
            pos: tag_pos,
        });
        scanner.advance_to(tag_end);
    }

    Ok(atoms)
}

/// テキストを改行（`\n`または`\r\n`）ごとに分割し、`Atom::Text`として追加する。
fn push_text_atoms(atoms: &mut Vec<Atom>, text: &str) {
    let mut rest = text;
    loop {
        match rest.find('\n') {
            None => {
                if !rest.is_empty() {
                    atoms.push(Atom::Text {
                        content: rest.to_string(),
                        newline: None,
                    });
                }
                break;
            }
            Some(idx) => {
                let (before_nl, newline) = if idx > 0 && rest.as_bytes()[idx - 1] == b'\r' {
                    (&rest[..idx - 1], "\r\n")
                } else {
                    (&rest[..idx], "\n")
                };
                atoms.push(Atom::Text {
                    content: before_nl.to_string(),
                    newline: Some(newline),
                });
                rest = &rest[idx + 1..];
            }
        }
    }
}

/// Pass 2: 行単位でスタンドアロンタグ（BR-7.1/BR-7.2）を判定し、該当行の空白・改行を除去する。
/// パーシャルタグについては、除去前の行頭空白を`indent`として`Item::Tag`に埋め込む。
fn apply_standalone_trimming(atoms: Vec<Atom>) -> Vec<Item> {
    // 改行を持つTextAtomの直後を行の区切りとして、[start, end)の行範囲一覧を作る。
    let mut lines: Vec<(usize, usize)> = Vec::new();
    let mut line_start = 0usize;
    for (i, atom) in atoms.iter().enumerate() {
        if let Atom::Text {
            newline: Some(_), ..
        } = atom
        {
            lines.push((line_start, i + 1));
            line_start = i + 1;
        }
    }
    if line_start < atoms.len() {
        lines.push((line_start, atoms.len()));
    }

    let mut items: Vec<Item> = atoms
        .into_iter()
        .map(|a| match a {
            Atom::Text { content, newline } => Item::Text { content, newline },
            Atom::Tag { parsed, pos } => Item::Tag {
                parsed,
                pos,
                indent: String::new(),
            },
        })
        .collect();

    for (start, end) in lines {
        let slice = &items[start..end];

        let has_tag = slice.iter().any(|a| matches!(a, Item::Tag { .. }));
        let all_tags_block_type = slice.iter().all(|a| match a {
            Item::Tag { parsed, .. } => parsed.is_block_type(),
            Item::Text { .. } => true,
        });
        let all_text_whitespace = slice.iter().all(|a| match a {
            Item::Text { content, .. } => content.chars().all(|c| c == ' ' || c == '\t'),
            Item::Tag { .. } => true,
        });

        if !(has_tag && all_tags_block_type && all_text_whitespace) {
            continue;
        }

        // インデント採取: パーシャルタグの直前がTextであれば、その内容をindentとする
        // （除去される前に確定させておく必要がある）。
        for i in start..end {
            if matches!(&items[i], Item::Tag { parsed: ParsedTag::Partial { .. }, .. }) {
                let indent_text = if i > start {
                    match &items[i - 1] {
                        Item::Text { content, .. } => content.clone(),
                        Item::Tag { .. } => String::new(),
                    }
                } else {
                    String::new()
                };
                if let Item::Tag { indent, .. } = &mut items[i] {
                    *indent = indent_text;
                }
            }
        }

        // スタンドアロン行: 前後の空白と改行を出力から除去する（BR-7.1）。
        for i in start..end {
            if let Item::Text { content, newline } = &mut items[i] {
                content.clear();
                *newline = None;
            }
        }
    }

    items
}

/// Pass 3: クリーニング済みの`Item`列からセクション木を構築する。
fn build_tree(items: Vec<Item>) -> Result<Vec<Node>, ParseError> {
    let mut stack: Vec<(String, bool, Vec<Node>, SourcePosition)> = Vec::new();
    let mut current: Vec<Node> = Vec::new();

    for item in items {
        match item {
            Item::Text { content, newline } => {
                let mut text = content;
                if let Some(nl) = newline {
                    text.push_str(nl);
                }
                if !text.is_empty() {
                    // 隣接するテキスト断片は1つのNode::Textにまとめる。
                    match current.last_mut() {
                        Some(Node::Text(existing)) => existing.push_str(&text),
                        _ => current.push(Node::Text(text)),
                    }
                }
            }
            Item::Tag {
                parsed,
                pos,
                indent,
            } => match parsed {
                ParsedTag::Variable { name, escape } => {
                    current.push(Node::Variable { name, escape, pos });
                }
                ParsedTag::SectionStart { name } => {
                    let parent = std::mem::take(&mut current);
                    stack.push((name, false, parent, pos));
                }
                ParsedTag::Inverted { name } => {
                    let parent = std::mem::take(&mut current);
                    stack.push((name, true, parent, pos));
                }
                ParsedTag::SectionEnd { name } => match stack.pop() {
                    None => {
                        return Err(to_parse_error(
                            ParseErrorKind::UnbalancedSection { name },
                            pos,
                        ));
                    }
                    Some((open_name, inverted, parent, start_pos)) => {
                        if open_name != name {
                            return Err(to_parse_error(
                                ParseErrorKind::UnbalancedSection { name },
                                pos,
                            ));
                        }
                        let children = std::mem::replace(&mut current, parent);
                        current.push(Node::Section {
                            name: open_name,
                            inverted,
                            children,
                            pos: start_pos,
                        });
                    }
                },
                ParsedTag::Partial { name } => {
                    current.push(Node::Partial { name, indent, pos });
                }
                ParsedTag::Comment => {}
                ParsedTag::DelimChange { .. } => {}
            },
        }
    }

    if let Some((name, _, _, pos)) = stack.pop() {
        return Err(to_parse_error(
            ParseErrorKind::UnbalancedSection { name },
            pos,
        ));
    }

    Ok(current)
}

struct Scanner<'a> {
    src: &'a str,
    pos: usize,
    line: usize,
    column: usize,
}

impl<'a> Scanner<'a> {
    fn new(src: &'a str) -> Self {
        Self {
            src,
            pos: 0,
            line: 1,
            column: 1,
        }
    }

    fn position_at(&self, target: usize) -> SourcePosition {
        let mut line = self.line;
        let mut column = self.column;
        for ch in self.src[self.pos..target].chars() {
            if ch == '\n' {
                line += 1;
                column = 1;
            } else {
                column += 1;
            }
        }
        SourcePosition { line, column }
    }

    fn advance_to(&mut self, target: usize) {
        let p = self.position_at(target);
        self.line = p.line;
        self.column = p.column;
        self.pos = target;
    }
}

fn find_from(src: &str, from: usize, needle: &str) -> Option<usize> {
    src[from..].find(needle).map(|i| from + i)
}

/// `tag_start`位置にあるタグを解析し、(タグ全体の終端位置, 解析結果)を返す。
fn scan_tag(
    src: &str,
    tag_start: usize,
    open: &str,
    close: &str,
) -> Result<(usize, ParsedTag), ParseErrorKind> {
    let after_open = tag_start + open.len();

    // トリプルマスタッシュ（{{{ }}}）はデフォルトデリミタでのみ有効。
    if open == "{{" && src[after_open..].starts_with('{') {
        let content_start = after_open + 1;
        let mut search_close = String::from("}");
        search_close.push_str(close);
        return match src[content_start..].find(&search_close) {
            None => Err(ParseErrorKind::UnexpectedEof),
            Some(rel) => {
                let close_pos = content_start + rel;
                let tag_end = close_pos + search_close.len();
                let name = src[content_start..close_pos].trim().to_string();
                if name.is_empty() {
                    return Err(ParseErrorKind::EmptyTagName);
                }
                Ok((tag_end, ParsedTag::Variable { name, escape: false }))
            }
        };
    }

    match src[after_open..].find(close) {
        None => Err(ParseErrorKind::UnexpectedEof),
        Some(rel) => {
            let close_pos = after_open + rel;
            let tag_end = close_pos + close.len();
            let raw = src[after_open..close_pos].trim();
            let parsed = parse_tag_content(raw)?;
            Ok((tag_end, parsed))
        }
    }
}

fn parse_tag_content(raw: &str) -> Result<ParsedTag, ParseErrorKind> {
    if raw.is_empty() {
        return Err(ParseErrorKind::EmptyTagName);
    }

    let first = raw.chars().next().unwrap();
    let rest_after_sigil = || raw[first.len_utf8()..].trim().to_string();

    let parsed = match first {
        '#' => ParsedTag::SectionStart {
            name: rest_after_sigil(),
        },
        '^' => ParsedTag::Inverted {
            name: rest_after_sigil(),
        },
        '/' => ParsedTag::SectionEnd {
            name: rest_after_sigil(),
        },
        '>' => ParsedTag::Partial {
            name: rest_after_sigil(),
        },
        '!' => ParsedTag::Comment,
        '&' => ParsedTag::Variable {
            name: rest_after_sigil(),
            escape: false,
        },
        '=' => {
            if raw.len() < 2 || !raw.ends_with('=') {
                return Err(ParseErrorKind::UnknownDelimiterSyntax);
            }
            let inner = raw[1..raw.len() - 1].trim();
            let parts: Vec<&str> = inner.split_whitespace().collect();
            if parts.len() != 2 {
                return Err(ParseErrorKind::UnknownDelimiterSyntax);
            }
            ParsedTag::DelimChange {
                open: parts[0].to_string(),
                close: parts[1].to_string(),
            }
        }
        _ => ParsedTag::Variable {
            name: raw.trim().to_string(),
            escape: true,
        },
    };

    let name_empty = match &parsed {
        ParsedTag::SectionStart { name }
        | ParsedTag::Inverted { name }
        | ParsedTag::SectionEnd { name }
        | ParsedTag::Partial { name }
        | ParsedTag::Variable { name, .. } => name.is_empty(),
        _ => false,
    };
    if name_empty {
        return Err(ParseErrorKind::EmptyTagName);
    }

    Ok(parsed)
}

fn to_parse_error(kind: ParseErrorKind, pos: SourcePosition) -> ParseError {
    let message = describe_parse_error(&kind);
    ParseError {
        kind,
        line: pos.line,
        column: pos.column,
        message,
    }
}

fn describe_parse_error(kind: &ParseErrorKind) -> String {
    match kind {
        ParseErrorKind::UnexpectedEof => {
            "unexpected end of template while scanning a tag".to_string()
        }
        ParseErrorKind::UnbalancedSection { name } => {
            format!("unbalanced section: {name}")
        }
        ParseErrorKind::UnknownDelimiterSyntax => "invalid delimiter change syntax".to_string(),
        ParseErrorKind::EmptyTagName => "tag name must not be empty".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse_ok(template: &str) -> Vec<Node> {
        parse(template).unwrap_or_else(|e| panic!("parse failed: {e:?}"))
    }

    #[test]
    fn plain_text_only() {
        let nodes = parse_ok("hello world");
        assert_eq!(nodes, vec![Node::Text("hello world".to_string())]);
    }

    #[test]
    fn escaped_variable() {
        let nodes = parse_ok("Hello, {{name}}!");
        assert_eq!(nodes.len(), 3);
        assert!(matches!(&nodes[1], Node::Variable { name, escape: true, .. } if name == "name"));
    }

    #[test]
    fn unescaped_variable_triple() {
        let nodes = parse_ok("{{{raw}}}");
        assert!(matches!(&nodes[0], Node::Variable { name, escape: false, .. } if name == "raw"));
    }

    #[test]
    fn unescaped_variable_amp() {
        let nodes = parse_ok("{{&raw}}");
        assert!(matches!(&nodes[0], Node::Variable { name, escape: false, .. } if name == "raw"));
    }

    #[test]
    fn section_nesting() {
        let nodes = parse_ok("{{#a}}{{#b}}x{{/b}}{{/a}}");
        assert_eq!(nodes.len(), 1);
        match &nodes[0] {
            Node::Section { name, children, inverted, .. } => {
                assert_eq!(name, "a");
                assert!(!inverted);
                assert_eq!(children.len(), 1);
                match &children[0] {
                    Node::Section { name, children, .. } => {
                        assert_eq!(name, "b");
                        assert_eq!(children, &vec![Node::Text("x".to_string())]);
                    }
                    other => panic!("unexpected node: {other:?}"),
                }
            }
            other => panic!("unexpected node: {other:?}"),
        }
    }

    #[test]
    fn inverted_section() {
        let nodes = parse_ok("{{^a}}x{{/a}}");
        assert!(matches!(&nodes[0], Node::Section { inverted: true, .. }));
    }

    #[test]
    fn unbalanced_section_missing_end() {
        let err = parse("{{#a}}x").unwrap_err();
        assert!(matches!(err.kind, ParseErrorKind::UnbalancedSection { .. }));
    }

    #[test]
    fn unbalanced_section_mismatched_end() {
        let err = parse("{{#a}}x{{/b}}").unwrap_err();
        assert!(matches!(err.kind, ParseErrorKind::UnbalancedSection { .. }));
    }

    #[test]
    fn unexpected_eof_in_tag() {
        let err = parse("{{name").unwrap_err();
        assert!(matches!(err.kind, ParseErrorKind::UnexpectedEof));
    }

    #[test]
    fn comment_standalone_removed() {
        let nodes = parse_ok("before\n{{! comment }}\nafter");
        assert_eq!(nodes, vec![Node::Text("before\nafter".to_string())]);
    }

    #[test]
    fn comment_inline_not_trimmed() {
        let nodes = parse_ok("a {{! c }} b");
        assert_eq!(nodes, vec![Node::Text("a  b".to_string())]);
    }

    #[test]
    fn section_standalone_trims_whitespace_lines() {
        let nodes = parse_ok("{{#a}}\nx\n{{/a}}\n");
        match &nodes[0] {
            Node::Section { children, .. } => {
                assert_eq!(children, &vec![Node::Text("x\n".to_string())]);
            }
            other => panic!("unexpected node: {other:?}"),
        }
    }

    #[test]
    fn delimiter_change() {
        let nodes = parse_ok("{{=<% %>=}}<%name%>{{literal}}");
        assert!(matches!(&nodes[0], Node::Variable { name, .. } if name == "name"));
        // 元のデリミタ({{ }})はもはやタグとして解釈されず、リテラルテキストになる
        assert_eq!(nodes[1], Node::Text("{{literal}}".to_string()));
    }

    #[test]
    fn delimiter_change_invalid_syntax() {
        let err = parse("{{=only=}}").unwrap_err();
        assert!(matches!(err.kind, ParseErrorKind::UnknownDelimiterSyntax));
    }

    #[test]
    fn partial_standalone_captures_indent() {
        let nodes = parse_ok("  {{> p}}\n");
        assert!(matches!(&nodes[0], Node::Partial { name, indent, .. } if name == "p" && indent == "  "));
    }

    #[test]
    fn empty_tag_name_errors() {
        let err = parse("{{}}").unwrap_err();
        assert!(matches!(err.kind, ParseErrorKind::EmptyTagName));
    }

    #[test]
    fn position_tracking() {
        let nodes = parse_ok("a\nb{{name}}");
        match &nodes[1] {
            Node::Variable { pos, .. } => {
                assert_eq!(pos.line, 2);
                assert_eq!(pos.column, 2);
            }
            other => panic!("unexpected node: {other:?}"),
        }
    }

    #[test]
    fn multiple_block_tags_on_one_line_are_standalone() {
        let nodes = parse_ok("{{#a}}{{/a}}\nafter");
        assert_eq!(nodes.len(), 2);
        assert!(matches!(&nodes[0], Node::Section { name, .. } if name == "a"));
        assert_eq!(nodes[1], Node::Text("after".to_string()));
    }

    #[test]
    fn variable_tag_disqualifies_line_from_standalone() {
        // 同じ行に変数タグがあると、他のブロックタグも含めて行全体がインライン扱いになる。
        let nodes = parse_ok("  {{name}}  {{! c }}\nafter");
        // 変数タグの存在により、コメントタグの前後空白は除去されない。
        let joined: String = nodes
            .iter()
            .map(|n| match n {
                Node::Text(t) => t.clone(),
                Node::Variable { .. } => "<var>".to_string(),
                _ => String::new(),
            })
            .collect();
        assert_eq!(joined, "  <var>  \nafter");
    }

    #[test]
    fn crlf_standalone_tags_are_trimmed() {
        let nodes = parse_ok("|\r\n{{! c }}\r\n|");
        assert_eq!(nodes, vec![Node::Text("|\r\n|".to_string())]);
    }
}
