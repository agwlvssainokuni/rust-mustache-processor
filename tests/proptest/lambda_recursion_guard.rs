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

//! Property: ラムダの再帰レンダリングもネスト深度ガードで終端する（Invariant、v0.2.0追加）。
//!
//! 自身を呼び出すテンプレート文字列を返すラムダに対し、レンダリングは無限再帰せず
//! 有限時間で`MaxNestingDepthExceeded`を返すことを検証する（BR-9.3）。

use std::rc::Rc;

use mustache_processor::Mustache;
use mustache_processor::error::Error;
use mustache_processor::value::{Map, Value};
use proptest::prelude::*;

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    #[test]
    fn self_referential_lambda_terminates(name in "[a-z]{1,8}") {
        let mut data = Map::new();
        let tag_name = name.clone();
        data.insert(
            name.as_str(),
            Value::Lambda(Rc::new(move |_: &str| format!("{{{{{tag_name}}}}}"))),
        );

        let mut template = String::from("{{");
        template.push_str(&name);
        template.push_str("}}");

        let result = Mustache::new().render_str(&template, &Value::Map(data));
        prop_assert!(matches!(result, Err(Error::Render(_))));
    }
}
