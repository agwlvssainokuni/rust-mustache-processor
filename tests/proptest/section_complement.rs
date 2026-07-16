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

//! Property: セクション/逆セクションの相補性（Invariant）。

use crate::support::arb_value;
use mustache_processor::Mustache;
use mustache_processor::value::{Map, Value};
use proptest::prelude::*;

proptest! {
    #![proptest_config(ProptestConfig::with_cases(256))]

    #[test]
    fn section_and_inverted_are_complementary(v in arb_value()) {
        let mut data = Map::new();
        data.insert("key", v);
        let mustache = Mustache::new();
        let out = mustache
            .render_str("{{#key}}A{{/key}}{{^key}}B{{/key}}", &Value::Map(data))
            .unwrap();
        let has_a = out.contains('A');
        let has_b = out.contains('B');
        prop_assert!(has_a ^ has_b, "expected exactly one of A/B, got {out:?}");
    }
}
