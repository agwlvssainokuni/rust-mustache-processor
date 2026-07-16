# Integration Test Instructions — rust-mustache-processor

## ユニット間結合の構造

`unit-of-work-dependency.md`の通り、本プロジェクトは単一Cargoパッケージ内のlib+binパターンであり、依存方向は`cli → core-engine`の単方向のみ（コンパイラにより強制）。cliはcore-engineの公開API（`Mustache`, `Template`, `Value`, `PartialResolver`, `DirectoryPartialResolver`, エラー型）のみを利用する。

## 実行コマンド

```bash
# 統合テストを含む全テストの一括実行
cargo test

# core-engineの公式spec conformanceテスト（integration test）のみ
cargo test --test spec

# core-engineのプロパティベーステスト（integration test）のみ
cargo test --test proptest
```

## ユニット間結合を検証しているテスト

真の意味での「ユニット間結合（core-engine ⇄ cli）」を検証しているのは、cliの`src/cli/mod.rs`内の`run_inner`/`run`を通したテスト群（7件）である。これらは以下を実際に結合させて検証する:

- `cli::args`（引数解析）→ `cli::io`（ファイルI/O）→ `cli::data_loader`（JSON変換）→ **core-engineの`Mustache`公開API**（パース・レンダリング）→ `cli::io`（出力書き出し）

具体的なテストケース:
- `run_inner_renders_single_template_to_file`: 単一テンプレート+JSONデータの一気通貫レンダリング
- `run_inner_concatenates_multiple_templates_in_order`: 複数テンプレートのprocess-then-cat処理
- `run_inner_uses_per_template_partials_dir`: core-engineの`DirectoryPartialResolver`をcli側がテンプレートごとに構築し、パーシャル解決が正しく機能することの確認
- `run_inner_fails_atomically_when_one_template_errors`: core-engine側のエラー（`mustache_processor::error::Error`）がcli側の`CliError::Mustache`に正しく伝播し、出力が一切書き出されないことの確認

## 公式spec準拠テスト（`tests/spec/`）

`tests/spec/`は、公式mustache/specリポジトリのJSONフィクスチャ（`tests/spec/fixtures/`、6モジュール136ケース）を用いて、core-engineライブラリの公開API（`Mustache::render_str`）が外部仕様と一致することを検証する統合テストである。ユニット間結合ではなく、core-engineライブラリと外部仕様との整合性を検証するものだが、Cargoの`tests/`ディレクトリ機構を用いる点で技術的には同種の統合テストに分類される。

```bash
cargo test --test spec
```

## 手動での結合確認（CLIエンドツーエンド）

自動テストとしては実装していないが、`cargo run --bin mustache`で実際のバイナリを起動し、ファイルシステム・標準入出力を介した完全なエンドツーエンドの動作を確認できる（Code Generation Step6で実施済み）:

```bash
# ビルド
cargo build --bin mustache

# 実行例
echo '{"name": "World"}' > /tmp/data.json
echo 'Hello, {{name}}!' > /tmp/template.tmpl
./target/debug/mustache /tmp/template.tmpl --data /tmp/data.json
# => Hello, World!
```

将来、CLIバイナリをサブプロセスとして起動する形のend-to-endテスト（`assert_cmd`クレート等を利用）を追加する余地はあるが、現時点では`run_inner`ベースの統合テストで十分なカバレッジが得られていると判断し、v1スコープでは追加していない（YAGNI）。
