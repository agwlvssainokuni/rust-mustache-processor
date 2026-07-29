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

//! 公式mustache/spec（<https://github.com/mustache/spec>）のJSONフィクスチャを読み込み、
//! 必須モジュール（comments/delimiters/interpolation/inverted/partials/sections）について
//! `Mustache::render_str`の出力が期待値と一致することを検証する。

use std::collections::HashMap;
use std::path::PathBuf;

use mustache_processor::Mustache;
use mustache_processor::partial::PartialResolver;
use mustache_processor::value::Value;

struct FixturePartialResolver(HashMap<String, String>);

impl PartialResolver for FixturePartialResolver {
    fn resolve(&self, name: &str) -> Option<String> {
        self.0.get(name).cloned()
    }
}

fn fixtures_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/spec/fixtures")
}

fn run_module(module: &str) {
    let path = fixtures_dir().join(format!("{module}.json"));
    let content = std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("failed to read fixture {}: {e}", path.display()));
    let doc: serde_json::Value = serde_json::from_str(&content)
        .unwrap_or_else(|e| panic!("failed to parse fixture {}: {e}", path.display()));

    let tests = doc["tests"].as_array().expect("tests array");
    let mut failures = Vec::new();

    for case in tests {
        let name = case["name"].as_str().unwrap_or("<unnamed>");
        let template = case["template"].as_str().expect("template field");
        let expected = case["expected"].as_str().expect("expected field");
        let data = Value::from_serialize(&case["data"])
            .unwrap_or_else(|e| panic!("[{module}/{name}] failed to convert data: {e}"));

        let mustache = match case.get("partials").and_then(|p| p.as_object()) {
            Some(partials) => {
                let map: HashMap<String, String> = partials
                    .iter()
                    .map(|(k, v)| (k.clone(), v.as_str().unwrap_or_default().to_string()))
                    .collect();
                Mustache::new().with_partial_resolver(Box::new(FixturePartialResolver(map)))
            }
            None => Mustache::new(),
        };

        match mustache.render_str(template, &data) {
            Ok(actual) if actual == expected => {}
            Ok(actual) => failures.push(format!(
                "[{module}/{name}] mismatch:\n  template: {template:?}\n  expected: {expected:?}\n  actual:   {actual:?}"
            )),
            Err(e) => failures.push(format!(
                "[{module}/{name}] render error: {e}\n  template: {template:?}"
            )),
        }
    }

    assert!(
        failures.is_empty(),
        "{} failure(s) in module '{module}':\n{}",
        failures.len(),
        failures.join("\n\n")
    );
}

#[test]
fn comments() {
    run_module("comments");
}

#[test]
fn delimiters() {
    run_module("delimiters");
}

#[test]
fn interpolation() {
    run_module("interpolation");
}

#[test]
fn inverted() {
    run_module("inverted");
}

#[test]
fn partials() {
    run_module("partials");
}

#[test]
fn sections() {
    run_module("sections");
}

#[test]
fn inheritance() {
    // ~inheritance.jsonのテストデータは通常のJSON値のみで、コード実行を伴わないため
    // 既存のrun_moduleでそのまま検証できる。
    run_module("~inheritance");
}

#[test]
fn dynamic_names() {
    // ~dynamic-names.jsonも同様に通常のJSON値のみで検証できる。
    run_module("~dynamic-names");
}

/// `~lambdas.json`のテストケース。
///
/// 各テストケースのラムダは、公式フィクスチャの`data.lambda.ruby`（Rubyの`proc`）で
/// 定義された挙動を手動でRustのクロージャとして再実装したものである
/// （JSONフィクスチャのRuby/JSソースコードをRust側で動的評価することはできないため）。
/// 対応関係は`tests/spec/fixtures/~lambdas.json`の各テストケースを参照。
#[test]
fn lambdas() {
    use std::cell::Cell;
    use std::rc::Rc;

    use mustache_processor::value::Map;

    let cases: Vec<(&str, &str, &str, Value)> = vec![
        (
            "Interpolation",
            "Hello, {{lambda}}!",
            "Hello, world!",
            {
                let mut m = Map::new();
                m.insert(
                    "lambda",
                    Value::Lambda(Rc::new(|_: &str| "world".to_string())),
                );
                Value::Map(m)
            },
        ),
        (
            "Interpolation - Expansion",
            "Hello, {{lambda}}!",
            "Hello, world!",
            {
                let mut m = Map::new();
                m.insert("planet", Value::String("world".to_string()));
                m.insert(
                    "lambda",
                    Value::Lambda(Rc::new(|_: &str| "{{planet}}".to_string())),
                );
                Value::Map(m)
            },
        ),
        (
            "Interpolation - Alternate Delimiters",
            "{{= | | =}}\nHello, (|&lambda|)!",
            "Hello, (|planet| => world)!",
            {
                let mut m = Map::new();
                m.insert("planet", Value::String("world".to_string()));
                m.insert(
                    "lambda",
                    Value::Lambda(Rc::new(|_: &str| "|planet| => {{planet}}".to_string())),
                );
                Value::Map(m)
            },
        ),
        (
            "Interpolation - Multiple Calls",
            "{{lambda}} == {{{lambda}}} == {{lambda}}",
            "1 == 2 == 3",
            {
                // BR-9.3b: 参照の都度呼び出されキャッシュされないことを検証する。
                let counter = Cell::new(0);
                let mut m = Map::new();
                m.insert(
                    "lambda",
                    Value::Lambda(Rc::new(move |_: &str| {
                        let n = counter.get() + 1;
                        counter.set(n);
                        n.to_string()
                    })),
                );
                Value::Map(m)
            },
        ),
        (
            "Escaping",
            "<{{lambda}}{{{lambda}}}",
            "<&gt;>",
            {
                let mut m = Map::new();
                m.insert("lambda", Value::Lambda(Rc::new(|_: &str| ">".to_string())));
                Value::Map(m)
            },
        ),
        (
            "Section",
            "<{{#lambda}}{{x}}{{/lambda}}>",
            "<yes>",
            {
                let mut m = Map::new();
                m.insert("x", Value::String("Error!".to_string()));
                m.insert(
                    "lambda",
                    Value::Lambda(Rc::new(|text: &str| {
                        if text == "{{x}}" {
                            "yes".to_string()
                        } else {
                            "no".to_string()
                        }
                    })),
                );
                Value::Map(m)
            },
        ),
        (
            "Section - Expansion",
            "<{{#lambda}}-{{/lambda}}>",
            "<-Earth->",
            {
                let mut m = Map::new();
                m.insert("planet", Value::String("Earth".to_string()));
                m.insert(
                    "lambda",
                    Value::Lambda(Rc::new(|text: &str| {
                        let mut s = text.to_string();
                        s.push_str("{{planet}}");
                        s.push_str(text);
                        s
                    })),
                );
                Value::Map(m)
            },
        ),
        (
            "Section - Alternate Delimiters",
            "{{= | | =}}<|#lambda|-|/lambda|>",
            "<-{{planet}} => Earth->",
            {
                let mut m = Map::new();
                m.insert("planet", Value::String("Earth".to_string()));
                m.insert(
                    "lambda",
                    Value::Lambda(Rc::new(|text: &str| {
                        let mut s = text.to_string();
                        s.push_str("{{planet}} => |planet|");
                        s.push_str(text);
                        s
                    })),
                );
                Value::Map(m)
            },
        ),
        (
            "Section - Multiple Calls",
            "{{#lambda}}FILE{{/lambda}} != {{#lambda}}LINE{{/lambda}}",
            "__FILE__ != __LINE__",
            {
                let mut m = Map::new();
                m.insert(
                    "lambda",
                    Value::Lambda(Rc::new(|text: &str| format!("__{text}__"))),
                );
                Value::Map(m)
            },
        ),
        (
            "Inverted Section",
            "<{{^lambda}}{{static}}{{/lambda}}>",
            "<>",
            {
                let mut m = Map::new();
                m.insert("static", Value::String("static".to_string()));
                // BR-9.5: 逆セクションのラムダは呼び出されないはず。誤って呼び出された場合に
                // 検出できるよう、判別可能な文字列を返すダミー実装にする。
                m.insert(
                    "lambda",
                    Value::Lambda(Rc::new(|_: &str| "WRONGLY_CALLED".to_string())),
                );
                Value::Map(m)
            },
        ),
    ];

    let mut failures = Vec::new();
    for (name, template, expected, data) in cases {
        let mustache = Mustache::new();
        match mustache.render_str(template, &data) {
            Ok(actual) if actual == expected => {}
            Ok(actual) => failures.push(format!(
                "[lambdas/{name}] mismatch:\n  template: {template:?}\n  expected: {expected:?}\n  actual:   {actual:?}"
            )),
            Err(e) => failures.push(format!(
                "[lambdas/{name}] render error: {e}\n  template: {template:?}"
            )),
        }
    }

    assert!(
        failures.is_empty(),
        "{} failure(s) in module 'lambdas':\n{}",
        failures.len(),
        failures.join("\n\n")
    );
}
