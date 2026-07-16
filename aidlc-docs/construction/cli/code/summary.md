# Code Generation Summary — cli

`cli-code-generation-plan.md`（全9ステップ）に基づき実装したcliユニットの生成物一覧、テスト構成、動作確認結果をまとめる。

## 生成物一覧

### バイナリ本体（`src/`, バイナリ名`mustache`）

| ファイル | 行数 | 内容 |
|---|---|---|
| `src/main.rs` | 20 | エントリーポイント。`cli::run(&argv)`の薄い呼び出し |
| `src/cli/mod.rs` | 260 | `CliError`とその`From`実装群、`run_inner`（オーケストレーション本体）、`run`（公開エントリー） |
| `src/cli/args.rs` | 240 | `RawArgs`（clap derive）→`CliArgs`（ドメイン型）変換、`CliArgsError`、`parse_args` |
| `src/cli/io.rs` | 257 | `TemplateSource`/`LoadedTemplate`/`IoError`、`read_templates`/`read_data`/`resolve_partials_dir`/`write_output` |
| `src/cli/data_loader.rs` | 252 | `DataFormat`/`DataLoaderError`、`detect_format`/`load`（JSON/YAML→core-engineの`Value`） |

`Cargo.toml`: `clap`（derive機能）を追加、`serde_json`をdev-dependencyから通常依存へ昇格、YAML実装として`serde_norway`（`serde_yaml`の非推奨化を受けての代替、詳細は`tech-stack-decisions.md`参照）を追加。

### テスト（`src/cli/`内`#[cfg(test)]`）

| モジュール | 種別 | 件数 |
|---|---|---|
| `args.rs` | ユニットテスト | 9件 |
| `io.rs` | ユニットテスト | 7件 |
| `data_loader.rs` | ユニットテスト（example-based） | 7件 |
| `data_loader.rs::tests::properties` | プロパティベーステスト（proptest） | 3件 |
| `mod.rs` | ユニットテスト（統合的な`run_inner`/`run`検証） | 7件 |

**cliユニット合計: 33テスト実行単位、全て成功**。標準入力を読む経路（`--template-stdin`, `--data`未指定時）はプロセス分離が必要なためexample-basedユニットテストの対象外とし、`cargo run`による手動確認で補完した。

### プロジェクト全体のテスト状況

`cargo test`実行時、以下が全て成功する:

- core-engine（`mustache_processor`ライブラリ）ユニットテスト: 72件
- cli（`mustache`バイナリ）ユニットテスト: 33件
- core-engineプロパティベーステスト（`tests/proptest/`）: 7件
- core-engine spec conformanceテスト（`tests/spec/`）: 6モジュール136ケース
- doctest: 1件

**合計: 119テスト実行単位、全て成功**。

## 動作確認

`cargo run --bin mustache`により、実際のテンプレートファイル+JSONデータファイルからのレンダリング（`Hello, {{name}}!` → `Hello, Rust!`等）、および引数未指定時のエラーメッセージ（`mustache: no template specified...`）・終了コード1を手動確認済み。

## Code Generation中に発見・修正した主な設計補正

いずれも`cli-code-generation-plan.md`の各Stepに詳細を記録済み:

1. **YAML実装クレートの変更**（Step1）: `serde_yaml`が作者により非推奨化されていたため、ユーザーに確認のうえメンテナンス継続中の後継クレート`serde_norway`に変更
2. **`CliArgsError`/`detect_format`のシグネチャ補正**（Step2）: `CliArgsError`へ`PartialEq`を付与する都合で`Clap(clap::Error)`を`Clap(String)`に変更。`detect_format`は`&CliArgs`を取ると`args`⇄`data_loader`モジュール間の循環依存が生じるため、必要なフィールドを直接引数に取る形に詳細化
3. **PBTテストの実装場所**（Step8）: `DataLoader`がcliバイナリクレート内部（`pub(crate)`）でありライブラリとして公開されていないため、当初計画していた`tests/proptest/`（外部統合テスト）からはアクセス不能と判明。`data_loader.rs`自身のユニットテストモジュール内に実装する方式に補正

## 未対応・対象外

- API Layer/Repository Layer/Frontend Components/Database Migration: cliはデータベース・Web API・UIを持たないCLIツールのため対象外
- Deployment Artifacts Generation（シングルバイナリ配布）: 将来のOperations Phaseで扱う

## 次のステップ

両ユニット（core-engine, cli）のCode Generationが完了したため、Build and Testステージ（全ユニット完了後、常時実行）へ進む。
