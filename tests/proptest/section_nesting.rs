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

//! Property: セクションの入れ子構造が保存される（Induction）。
//!
//! 深さ1のセクションでパースが成立するなら、それを子として含む深さN+1の
//! ネストしたセクションでも同様にパースが成立し、対応する開始・終了タグの
//! 組が正しく構造化されることを、公開APIから観測可能な形で検証する
//! （内部AST（`Node::Section`）は非公開のため、直接検査はできない）。

use mustache_processor::Mustache;
use mustache_processor::value::{Map, Value};
use proptest::prelude::*;

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    #[test]
    fn nested_sections_render_exactly_once(depth in 1usize..12) {
        let mut template = String::new();
        for i in 0..depth {
            template.push_str("{{#s");
            template.push_str(&i.to_string());
            template.push_str("}}");
        }
        template.push_str("INNER");
        for i in (0..depth).rev() {
            template.push_str("{{/s");
            template.push_str(&i.to_string());
            template.push_str("}}");
        }

        // 最も内側から外側に向けて、単一キーのMapを1階層ずつ積み上げる。
        let mut value = Value::Bool(true);
        for i in (0..depth).rev() {
            let mut m = Map::new();
            m.insert(format!("s{i}"), value);
            value = Value::Map(m);
        }

        let mustache = Mustache::new();
        let out = mustache.render_str(&template, &value).unwrap();
        prop_assert_eq!(out, "INNER");
    }
}
