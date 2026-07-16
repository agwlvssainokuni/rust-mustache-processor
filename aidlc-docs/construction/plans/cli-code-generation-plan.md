# Code Generation Plan — cli

## ユニットコンテキスト

- **要件**: FR-2（薄いラッパー）, FR-3, FR-5〜FR-8（`unit-of-work-story-map.md`）
- **依存**: core-engine（`mustache_processor`ライブラリ、公開APIのみ利用）
- **参照する承認済み成果物**:
  - `aidlc-docs/inception/application-design/components.md`, `component-methods.md`（CliArgs/IoController/DataLoader/CliRunnerの公開インターフェース）
  - `aidlc-docs/construction/cli/functional-design/`（domain-entities.md, business-rules.md, business-logic-model.md）
  - `aidlc-docs/construction/cli/nfr-requirements/`（tech-stack-decisions.md: clap derive API, serde_json/serde_yaml）
  - `aidlc-docs/construction/cli/nfr-design/`（nfr-design-patterns.md, logical-components.md: Result-based Core Logic, Atomic Output Buffering, Per-Template Partial Resolver Construction, Unified CliError）

## Plan Checklist

### Step 1: Project Structure Setup
- [x] `Cargo.toml`を更新: `clap`（`derive`機能）を`[dependencies]`に追加。`serde_json`を`[dev-dependencies]`から`[dependencies]`へ昇格（`tech-stack-decisions.md`の注記通り）
- [x] `src/cli/mod.rs`, `src/cli/args.rs`, `src/cli/io.rs`, `src/cli/data_loader.rs`を著作権ヘッダー付きの空ファイルとして作成
- [x] `src/main.rs`に`mod cli;`宣言を追加（Hello World雛形は後続Stepで置き換え）
- [x] `cargo build`成功を確認

**実装時の追加補正（要記録）**: `serde_yaml`を追加してビルドしたところ`serde_yaml v0.9.34+deprecated`と表示され、作者による非推奨化が判明。ユーザーに確認のうえ、`serde_yaml` 0.9系とAPI互換のメンテナンス継続中の後継クレート`serde_norway`に変更した。詳細は`tech-stack-decisions.md`（cli）に記録。

### Step 2: CliArgs（引数解析）
- [x] `src/cli/args.rs`: `clap::Parser`のderive APIで`RawArgs`（clap解析専用）を定義し、`CliArgs`（ドメイン型）へ変換する2段構成で実装（テンプレート位置引数`Vec<PathBuf>`、`--template-stdin`, `--data`, `--output`/`-o`, `--partials-dir`, `--strict`, `--format`）
- [x] `CliArgsError`（Clap/NoTemplateSpecified/TemplateAndStdinConflict/TemplateStdinAndDataStdinConflict/InvalidFormat）を定義
- [x] `parse_args(argv: &[String]) -> Result<CliArgs, CliArgsError>`を実装。BR-1.2/1.3/1.5は全てclap解析後の手動バリデーションで統一実装（`CliArgsError`の各バリアントで一貫したエラーメッセージを出すため、clapの`conflicts_with`は使わない方針に変更）
- [x] `src/cli/data_loader.rs`: `DataFormat`/`DataLoaderError`/`detect_format`/`load`を実装（`CliArgs`が`DataFormat`に依存するため、Step2でargs.rsと同時に実装。詳細はStep4参照）
- [x] ユニットテスト9件を実装し`cargo test --bin mustache args::`で全件成功を確認

**実装時の追加補正（要記録）**: `CliArgsError`に`PartialEq`を付与してテストしやすくするため、`clap::Error`（`PartialEq`未実装）をそのまま保持せず`Clap(String)`に変更。また`detect_format`は`&CliArgs`を引数に取ると`CliArgs`（`args.rs`）が`DataFormat`（`data_loader.rs`）に依存する一方で循環依存になるため、必要なフィールド（`explicit_format`, `data_path`）を直接引数に取る形に詳細化した。詳細は`domain-entities.md`（cli）に追記。

### Step 3: IoController（入出力）
- [x] `src/cli/io.rs`: `TemplateSource`, `LoadedTemplate`, `IoError`を定義
- [x] `read_templates`（BR-2.1〜2.3）、`read_data`（`--data`指定時はファイル、未指定時は標準入力）、`resolve_partials_dir`（BR-4.1〜4.3、テンプレートファイルごとの解決）、`write_output`（`--output`指定時はファイル、未指定時は標準出力）を実装
- [x] ユニットテスト7件を実装し`cargo test --bin mustache io::`で全件成功を確認（標準入力を使うケースはプロセス分離が必要なためexample-basedユニットテストの対象外とし、ファイル経由のケースで動作確認）

### Step 4: DataLoader（データ変換）
- [x] `src/cli/data_loader.rs`: `DataFormat`, `DataLoaderError`を定義（Step2でCliArgsの依存解消のため前倒し実装済み。詳細はStep2参照）
- [x] `detect_format`（BR-3.1〜3.3: `--format`最優先→拡張子→エラー）、`load`（`serde_json`/`serde_norway`でパースし`Value::from_serialize`で変換）を実装（Step2で前倒し実装済み）
- [x] ユニットテスト7件（形式判定優先順位、JSON/YAML往復、不正データのエラー）を実装し`cargo test --bin mustache data_loader::`で全件成功を確認

### Step 5: CliRunner（オーケストレーション）
- [x] `src/cli/mod.rs`: `CliError`とその`From`実装群（`logical-components.md`参照）を定義
- [x] `run_inner(argv) -> Result<(), CliError>`を実装（BR-5.1〜5.3, BR-6.1: process-then-cat、Atomic Output Buffering）
- [x] `run(argv) -> ExitCode`を実装（BR-7.1〜7.3: `run_inner`のラッパー、stderr出力・終了コード変換）
- [x] ユニットテスト7件（単一/複数テンプレートのレンダリング、テンプレートごとのパーシャルディレクトリ解決、エラー時の全体アトミック性、引数エラー伝播、ExitCode変換）を実装し`cargo test --bin mustache cli::tests::`で全件成功を確認

### Step 6: main.rs
- [x] `src/main.rs`を`cli::run(&std::env::args().collect::<Vec<_>>())`相当の薄い呼び出しに置き換える
- [x] `cargo build`成功（警告0件）を確認
- [x] `cargo run --bin mustache`で実際にテンプレート+JSONデータのレンダリング、および引数エラー時のstderr出力・終了コード1を手動確認

### Step 7: Unit Testing
- [ ] `args.rs`/`io.rs`/`data_loader.rs`/`mod.rs`内に`#[cfg(test)]`ユニットテスト（各BRの代表ケース: 複数テンプレート、`--template-stdin`と位置引数の競合、`--data`未指定時の標準入力、フォーマット判定優先順位、パーシャルディレクトリのファイルごと解決、エラー時の全体中断・出力なし等）

### Step 8: PBT Test Generation
- [ ] `tests/proptest/`にcli向けプロパティテストを追加（`business-logic-model.md`のTestable Properties: DataLoaderのJSON往復変換・YAML往復変換・形式判定の決定性、いずれもデフォルト256ケース）

### Step 9: Build Verification and Summary
- [ ] `cargo build --bin mustache`が警告なく完了することを確認
- [ ] `cargo test`（lib + cli統合分含む）が全て成功することを確認
- [ ] `aidlc-docs/construction/cli/code/summary.md`に生成物一覧・テスト構成をまとめる

**N/A（cliに該当なし）**: API Layer Generation, Repository Layer Generation, Frontend Components Generation, Database Migration Scripts — cliはデータベース・Web API・UIを持たないCLIツールのため対象外。Deployment Artifacts Generation（シングルバイナリ配布）は将来のOperations Phaseで扱う。
