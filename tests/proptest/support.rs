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

//! プロパティベーステスト共通のジェネレータ・ヘルパー関数。

use mustache_processor::value::{Map, Value};
use proptest::prelude::*;

/// スカラーの`Value`を生成するストラテジ。
fn arb_scalar() -> impl Strategy<Value = Value> {
    prop_oneof![
        Just(Value::Null),
        any::<bool>().prop_map(Value::Bool),
        any::<i32>().prop_map(|i| Value::Integer(i as i64)),
        (-1000i32..1000).prop_map(|i| Value::Float(f64::from(i) / 10.0)),
        "[a-zA-Z0-9 ]{0,10}".prop_map(Value::String),
    ]
}

/// 任意の深さ・種類を持つ`Value`を生成するストラテジ（Array/Mapを含む）。
pub fn arb_value() -> impl Strategy<Value = Value> {
    arb_scalar().prop_recursive(3, 20, 5, |inner| {
        prop_oneof![
            prop::collection::vec(inner.clone(), 0..5).prop_map(Value::Array),
            prop::collection::vec(("[a-z]{1,6}", inner), 0..5).prop_map(|entries| {
                let mut map = Map::new();
                for (k, v) in entries {
                    map.insert(k, v);
                }
                Value::Map(map)
            }),
        ]
    })
}

/// `renderer.rs`のBR-1.1エスケープ規則の逆変換（テスト検証専用）。
pub fn unescape_html(s: &str) -> String {
    s.replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&#39;", "'")
        .replace("&amp;", "&")
}
