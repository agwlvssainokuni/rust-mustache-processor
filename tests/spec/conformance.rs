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
