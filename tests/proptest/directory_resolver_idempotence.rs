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

//! Property: DirectoryPartialResolverの解決結果安定性（Idempotence）。

use mustache_processor::partial::{DirectoryPartialResolver, PartialResolver};
use proptest::prelude::*;
use std::fs;

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    #[test]
    fn directory_resolver_is_idempotent(content in "[a-zA-Z0-9 ]{0,40}") {
        let dir = std::env::temp_dir().join(format!(
            "mustache_processor_proptest_{}_dir_resolver_idem",
            std::process::id()
        ));
        fs::create_dir_all(&dir).unwrap();
        fs::write(dir.join("p.mustache"), &content).unwrap();

        let resolver = DirectoryPartialResolver::new(dir.clone());
        let r1 = resolver.resolve("p");
        let r2 = resolver.resolve("p");
        let r3 = resolver.resolve("p");

        fs::remove_dir_all(&dir).ok();

        prop_assert_eq!(&r1, &r2);
        prop_assert_eq!(&r1, &r3);
    }
}
