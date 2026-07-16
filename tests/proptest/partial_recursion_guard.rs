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

//! Property: 無限パーシャル再帰時の終端保証（Invariant）。
//!
//! 常に自身を指すパーシャルを解決する`PartialResolver`実装に対し、レンダリングは
//! 無限再帰・スタックオーバーフローせず、有限時間で`MaxNestingDepthExceeded`を
//! 返すことを検証する（BR-5.5削除に伴い、循環検出ではなく深度ガードが安全装置）。

use mustache_processor::error::Error;
use mustache_processor::partial::PartialResolver;
use mustache_processor::value::{Map, Value};
use mustache_processor::Mustache;
use proptest::prelude::*;

struct AlwaysSelfResolver;

impl PartialResolver for AlwaysSelfResolver {
    fn resolve(&self, name: &str) -> Option<String> {
        let mut s = String::from("{{> ");
        s.push_str(name);
        s.push_str("}}");
        Some(s)
    }
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    #[test]
    fn infinite_partial_recursion_terminates(name in "[a-z]{1,8}") {
        let mustache = Mustache::new().with_partial_resolver(Box::new(AlwaysSelfResolver));
        let mut template = String::from("{{> ");
        template.push_str(&name);
        template.push_str("}}");

        let result = mustache.render_str(&template, &Value::Map(Map::new()));
        prop_assert!(matches!(result, Err(Error::Render(_))));
    }
}
