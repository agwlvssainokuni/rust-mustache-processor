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

use crate::ast::{Node, SourcePosition};
use crate::error::{ParseError, ParseErrorKind};

/// テンプレート文字列からASTノード列を生成する。
pub(crate) fn parse(template: &str) -> Result<Vec<Node>, ParseError> {
    let mut scanner = Scanner::new(template);
    let mut open = String::from("{{");
    let mut close = String::from("}}");
    let mut stack: Vec<(String, bool, Vec<Node>, SourcePosition)> = Vec::new();
    let mut current: Vec<Node> = Vec::new();

    loop {
        let search_start = scanner.pos;
        let tag_start = match find_from(scanner.src, search_start, &open) {
            Some(p) => p,
            None => {
                let text = &scanner.src[search_start..];
                if !text.is_empty() {
                    current.push(Node::Text(text.to_string()));
                }
                break;
            }
        };

        let text_before = &scanner.src[search_start..tag_start];
        let tag_pos = scanner.position_at(tag_start);

        let (tag_end_raw, parsed) =
            scan_tag(scanner.src, tag_start, &open, &close).map_err(|kind| to_parse_error(kind, tag_pos))?;

        let is_block = matches!(
            parsed,
            ParsedTag::SectionStart { .. }
                | ParsedTag::Inverted { .. }
                | ParsedTag::SectionEnd { .. }
                | ParsedTag::Partial { .. }
                | ParsedTag::Comment
                | ParsedTag::DelimChange { .. }
        );

        let mut emit_text = text_before;
        let mut tag_end = tag_end_raw;
        let mut indent = String::new();

        if is_block {
            if let Some(left_ws_start) = standalone_left_ws_start(text_before) {
                let after = &scanner.src[tag_end_raw..];
                if let Some(right_ws_end) = standalone_right_ws_end(after) {
                    indent = text_before[left_ws_start..].to_string();
                    emit_text = &text_before[..left_ws_start];
                    tag_end = tag_end_raw + right_ws_end;
                }
            }
        }

        if !emit_text.is_empty() {
            current.push(Node::Text(emit_text.to_string()));
        }

        match parsed {
            ParsedTag::Variable { name, escape } => {
                current.push(Node::Variable {
                    name,
                    escape,
                    pos: tag_pos,
                });
            }
            ParsedTag::SectionStart { name } => {
                let parent = std::mem::take(&mut current);
                stack.push((name, false, parent, tag_pos));
            }
            ParsedTag::Inverted { name } => {
                let parent = std::mem::take(&mut current);
                stack.push((name, true, parent, tag_pos));
            }
            ParsedTag::SectionEnd { name } => match stack.pop() {
                None => {
                    return Err(to_parse_error(
                        ParseErrorKind::UnbalancedSection { name },
                        tag_pos,
                    ));
                }
                Some((open_name, inverted, parent, start_pos)) => {
                    if open_name != name {
                        return Err(to_parse_error(
                            ParseErrorKind::UnbalancedSection { name },
                            tag_pos,
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
                current.push(Node::Partial {
                    name,
                    indent,
                    pos: tag_pos,
                });
            }
            ParsedTag::Comment => {}
            ParsedTag::DelimChange {
                open: new_open,
                close: new_close,
            } => {
                open = new_open;
                close = new_close;
            }
        }

        scanner.advance_to(tag_end);
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

enum ParsedTag {
    Variable { name: String, escape: bool },
    SectionStart { name: String },
    Inverted { name: String },
    SectionEnd { name: String },
    Partial { name: String },
    Comment,
    DelimChange { open: String, close: String },
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

/// タグ直前のテキスト（行頭から）が空白のみであれば、その開始位置を返す（BR-7.1）。
fn standalone_left_ws_start(text_before: &str) -> Option<usize> {
    let tail_start = match text_before.rfind('\n') {
        Some(idx) => idx + 1,
        None => 0,
    };
    let tail = &text_before[tail_start..];
    if tail.chars().all(|c| c == ' ' || c == '\t') {
        Some(tail_start)
    } else {
        None
    }
}

/// タグ直後のテキスト（次の改行またはEOFまで）が空白のみであれば、
/// 改行を含めて消費すべき終端オフセット（`after`先頭からの相対位置）を返す（BR-7.1）。
fn standalone_right_ws_end(after: &str) -> Option<usize> {
    match after.find('\n') {
        Some(idx) => {
            if after[..idx].chars().all(|c| c == ' ' || c == '\t') {
                Some(idx + 1)
            } else {
                None
            }
        }
        None => {
            if after.chars().all(|c| c == ' ' || c == '\t') {
                Some(after.len())
            } else {
                None
            }
        }
    }
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
        ParseErrorKind::UnknownDelimiterSyntax => {
            "invalid delimiter change syntax".to_string()
        }
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
        assert_eq!(
            nodes,
            vec![
                Node::Text("before\n".to_string()),
                Node::Text("after".to_string()),
            ]
        );
    }

    #[test]
    fn comment_inline_not_trimmed() {
        let nodes = parse_ok("a {{! c }} b");
        assert_eq!(
            nodes,
            vec![
                Node::Text("a ".to_string()),
                Node::Text(" b".to_string()),
            ]
        );
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
}
