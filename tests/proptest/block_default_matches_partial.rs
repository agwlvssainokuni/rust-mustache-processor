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

//! Property: オーバーライドされないブロックはデフォルト内容と一致する（Invariant、v0.2.0追加）。
//!
//! `{{<parent}}`の本体に同名の`{{$block}}`を含まない場合、レンダリング結果は
//! 親テンプレート単体（`{{<parent}}`を使わず直接パーシャルとして）をレンダリングした
//! 結果と一致することを検証する（BR-10.3）。

use std::collections::HashMap;

use mustache_processor::Mustache;
use mustache_processor::partial::PartialResolver;
use mustache_processor::value::{Map, Value};
use proptest::prelude::*;

struct MapResolver(HashMap<String, String>);

impl PartialResolver for MapResolver {
    fn resolve(&self, name: &str) -> Option<String> {
        self.0.get(name).cloned()
    }
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(128))]

    #[test]
    fn unoverridden_block_matches_plain_partial(
        block_name in "[a-z]{1,8}",
        default_content in "[a-zA-Z0-9 ]{0,20}",
    ) {
        let mut resolver = HashMap::new();
        resolver.insert(
            "parent".to_string(),
            format!("{{{{${block_name}}}}}{default_content}{{{{/{block_name}}}}}"),
        );

        let via_parent = Mustache::new()
            .with_partial_resolver(Box::new(MapResolver(resolver.clone())))
            .render_str("{{<parent}}{{/parent}}", &Value::Map(Map::new()))
            .unwrap();

        let via_plain_partial = Mustache::new()
            .with_partial_resolver(Box::new(MapResolver(resolver)))
            .render_str("{{> parent}}", &Value::Map(Map::new()))
            .unwrap();

        prop_assert_eq!(via_parent, via_plain_partial);
    }
}
