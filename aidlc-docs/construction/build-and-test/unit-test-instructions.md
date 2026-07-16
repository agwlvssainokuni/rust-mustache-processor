# Unit Test Instructions — rust-mustache-processor

## テスト構成の全体像

| ユニット | 種別 | 場所 | 件数 |
|---|---|---|---|
| core-engine | ユニットテスト（`#[cfg(test)]`） | `src/*.rs`内 | 72件 |
| core-engine | doctest | `src/lib.rs`クレートdoc | 1件 |
| cli | ユニットテスト（`#[cfg(test)]`） | `src/cli/*.rs`内 | 33件（うちproptest 3件） |

## 実行コマンド

### 全ユニットテスト一括実行

```bash
cargo test
```

lib（core-engine）、bin（cli）、`tests/`配下の統合テスト（後述）、doctestを全て実行する。

### ユニット単位での実行

```bash
# core-engineのユニットテストのみ
cargo test --lib

# cliのユニットテストのみ
cargo test --bin mustache

# doctestのみ
cargo test --doc
```

### モジュール単位での絞り込み

```bash
# core-engineのパーサーのみ
cargo test --lib parser::

# core-engineのレンダラーのみ
cargo test --lib renderer::

# core-engineのValue/Mapのみ
cargo test --lib value::

# cliの引数解析のみ
cargo test --bin mustache args::

# cliのIoControllerのみ
cargo test --bin mustache io::

# cliのDataLoaderのみ
cargo test --bin mustache data_loader::

# cliのCliRunner（run_inner/run）のみ
cargo test --bin mustache cli::tests::
```

## テスト内容の要約

### core-engine（`mustache_processor`ライブラリ）

- `value.rs`（14件）: `is_truthy`各パターン、`get`/`iter`、`Map`の挿入順序保持、`from_serialize`（プリミティブ/struct/Vec/Map/Option/ネスト構造）
- `parser.rs`（17件）: 各タグ種別、デリミタ変更、スタンドアロン行トリミング（複数タグ・`\r\n`対応）、構文エラー
- `renderer.rs`（31件）: エスケープ、セクション各パターン、暗黙のイテレータ`.`、ドット区切り名前、strictモード、パーシャル（自己再帰・循環時の深度ガード・インデント適用）、ネスト深度超過
- `partial.rs`（2件）: `DirectoryPartialResolver`
- `lib.rs`（5件）: `Mustache`公開APIの統合テスト
- クレートdoc（1件）: 使用例のdoctest

### cli（`mustache`バイナリ）

- `args.rs`（9件）: 複数テンプレート、`--template-stdin`との競合、`--data`未指定時の扱い、フォーマット指定、各種オプション
- `io.rs`（7件）: テンプレート読み込み（複数ファイル・エラー）、データ読み込み、パーシャルディレクトリ解決（明示指定/ファイルごと/標準入力時のカレントディレクトリ）、出力書き出し
- `data_loader.rs`（10件、うちproptest 3件）: フォーマット判定優先順位、JSON/YAML読み込み、往復変換プロパティ、判定の決定性
- `mod.rs`（7件）: `run_inner`/`run`を通した統合的な検証（単一/複数テンプレートのレンダリング、テンプレートごとのパーシャルディレクトリ解決、エラー時の全体アトミック性、`ExitCode`変換）

## 既知の対象外事項

標準入力を読む経路（`--template-stdin`, `--data`未指定時の標準入力読み込み）は、プロセス分離が必要なためexample-basedユニットテストの対象外としている。実動作は`cargo run --bin mustache`による手動確認、および同等のファイル経由の経路（`resolve_partials_dir`のStdinケース等）のテストで補完済み（`cli/code/summary.md`参照）。
