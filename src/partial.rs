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

//! パーシャル名からテンプレート文字列を解決するための抽象化。

use std::fs;
use std::path::PathBuf;

/// パーシャル名からテンプレート文字列を取得するためのトレイト。
pub trait PartialResolver {
    /// パーシャル名からテンプレート文字列を取得する。未解決時は`None`を返す。
    fn resolve(&self, name: &str) -> Option<String>;
}

/// ディレクトリ配下の`{name}.mustache`ファイルからパーシャルを解決する標準実装。
pub struct DirectoryPartialResolver {
    base_dir: PathBuf,
}

impl DirectoryPartialResolver {
    /// パーシャル探索対象ディレクトリを指定して生成する。
    pub fn new(base_dir: impl Into<PathBuf>) -> Self {
        Self {
            base_dir: base_dir.into(),
        }
    }
}

impl PartialResolver for DirectoryPartialResolver {
    fn resolve(&self, name: &str) -> Option<String> {
        let path = self.base_dir.join(format!("{name}.mustache"));
        fs::read_to_string(path).ok()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn resolves_existing_partial_file() {
        let dir = std::env::temp_dir().join(format!(
            "mustache_processor_test_{}_{}",
            std::process::id(),
            "resolves_existing_partial_file"
        ));
        fs::create_dir_all(&dir).unwrap();
        let mut file = fs::File::create(dir.join("greeting.mustache")).unwrap();
        write!(file, "Hello, {{{{name}}}}!").unwrap();

        let resolver = DirectoryPartialResolver::new(dir.clone());
        assert_eq!(
            resolver.resolve("greeting"),
            Some("Hello, {{name}}!".to_string())
        );

        fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn missing_partial_file_returns_none() {
        let dir = std::env::temp_dir().join(format!(
            "mustache_processor_test_{}_{}",
            std::process::id(),
            "missing_partial_file_returns_none"
        ));
        fs::create_dir_all(&dir).unwrap();

        let resolver = DirectoryPartialResolver::new(dir.clone());
        assert_eq!(resolver.resolve("nonexistent"), None);

        fs::remove_dir_all(&dir).ok();
    }
}
