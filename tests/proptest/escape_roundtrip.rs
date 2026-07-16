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

//! Property: HTMLエスケープと非エスケープの相補性（Round-trip）。

use crate::support::unescape_html;
use mustache_processor::Mustache;
use mustache_processor::value::{Map, Value};
use proptest::prelude::*;

proptest! {
    #![proptest_config(ProptestConfig::with_cases(256))]

    #[test]
    fn escape_then_unescape_round_trips(s in "[-a-zA-Z0-9 &<>\"']{0,30}") {
        let mut data = Map::new();
        data.insert("v", Value::String(s.clone()));
        let mustache = Mustache::new();
        let escaped = mustache.render_str("{{v}}", &Value::Map(data)).unwrap();
        prop_assert_eq!(unescape_html(&escaped), s);
    }
}
