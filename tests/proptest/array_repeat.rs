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

//! Property: 配列セクションの繰り返し回数（Invariant）。

use mustache_processor::Mustache;
use mustache_processor::value::{Map, Value};
use proptest::prelude::*;

proptest! {
    #![proptest_config(ProptestConfig::with_cases(256))]

    #[test]
    fn array_section_repeats_body_per_element(len in 0usize..20) {
        let items: Vec<Value> = (0..len).map(|_| Value::Bool(true)).collect();
        let mut data = Map::new();
        data.insert("items", Value::Array(items));
        let mustache = Mustache::new();
        let out = mustache
            .render_str("{{#items}}X{{/items}}", &Value::Map(data))
            .unwrap();
        prop_assert_eq!(out, "X".repeat(len));
    }
}
