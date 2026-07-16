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
- [ ] `Cargo.toml`を更新: `clap`（`derive`機能）、`serde_yaml`を`[dependencies]`に追加。`serde_json`を`[dev-dependencies]`から`[dependencies]`へ昇格（`tech-stack-decisions.md`の注記通り）
- [ ] `src/cli/mod.rs`, `src/cli/args.rs`, `src/cli/io.rs`, `src/cli/data_loader.rs`を著作権ヘッダー付きの空ファイルとして作成
- [ ] `src/main.rs`を更新（現状のHello World雛形を置き換える準備）

### Step 2: CliArgs（引数解析）
- [ ] `src/cli/args.rs`: `clap::Parser`のderive APIで`CliArgs`を定義（テンプレート位置引数`Vec<PathBuf>`、`--template-stdin`, `--data`, `--output`/`-o`, `--partials-dir`, `--strict`, `--format`）
- [ ] `CliArgsError`（Clap/NoTemplateSpecified/TemplateAndStdinConflict/TemplateStdinAndDataStdinConflict/InvalidFormat）を定義
- [ ] `parse_args(argv: &[String]) -> Result<CliArgs, CliArgsError>`を実装。clapの`conflicts_with`で表現できる制約（BR-1.2）はderive属性で、`--data`未指定との組み合わせ（BR-1.5）はパース後の手動バリデーションで実装

### Step 3: IoController（入出力）
- [ ] `src/cli/io.rs`: `TemplateSource`, `LoadedTemplate`, `IoError`を定義
- [ ] `read_templates`（BR-2.1〜2.3）、`read_data`（`--data`指定時はファイル、未指定時は標準入力）、`resolve_partials_dir`（BR-4.1〜4.3、テンプレートファイルごとの解決）、`write_output`（`--output`指定時はファイル、未指定時は標準出力）を実装

### Step 4: DataLoader（データ変換）
- [ ] `src/cli/data_loader.rs`: `DataFormat`, `DataLoaderError`を定義
- [ ] `detect_format`（BR-3.1〜3.3: `--format`最優先→拡張子→エラー）、`load`（`serde_json`/`serde_yaml`でパースし`Value::from_serialize`で変換）を実装

### Step 5: CliRunner（オーケストレーション）
- [ ] `src/cli/mod.rs`: `CliError`とその`From`実装群（`logical-components.md`参照）を定義
- [ ] `run_inner(argv) -> Result<(), CliError>`を実装（BR-5.1〜5.3, BR-6.1: process-then-cat、Atomic Output Buffering）
- [ ] `run(argv) -> ExitCode`を実装（BR-7.1〜7.3: `run_inner`のラッパー、stderr出力・終了コード変換）

### Step 6: main.rs
- [ ] `src/main.rs`を`cli::run(&std::env::args().collect::<Vec<_>>())`相当の薄い呼び出しに置き換える

### Step 7: Unit Testing
- [ ] `args.rs`/`io.rs`/`data_loader.rs`/`mod.rs`内に`#[cfg(test)]`ユニットテスト（各BRの代表ケース: 複数テンプレート、`--template-stdin`と位置引数の競合、`--data`未指定時の標準入力、フォーマット判定優先順位、パーシャルディレクトリのファイルごと解決、エラー時の全体中断・出力なし等）

### Step 8: PBT Test Generation
- [ ] `tests/proptest/`にcli向けプロパティテストを追加（`business-logic-model.md`のTestable Properties: DataLoaderのJSON往復変換・YAML往復変換・形式判定の決定性、いずれもデフォルト256ケース）

### Step 9: Build Verification and Summary
- [ ] `cargo build --bin mustache`が警告なく完了することを確認
- [ ] `cargo test`（lib + cli統合分含む）が全て成功することを確認
- [ ] `aidlc-docs/construction/cli/code/summary.md`に生成物一覧・テスト構成をまとめる

**N/A（cliに該当なし）**: API Layer Generation, Repository Layer Generation, Frontend Components Generation, Database Migration Scripts — cliはデータベース・Web API・UIを持たないCLIツールのため対象外。Deployment Artifacts Generation（シングルバイナリ配布）は将来のOperations Phaseで扱う。
