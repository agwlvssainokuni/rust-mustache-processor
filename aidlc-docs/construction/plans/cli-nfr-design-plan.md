# NFR Design Plan — cli

`nfr-requirements.md`（cli）の決定事項を、具体的な設計パターン・論理コンポーネントに落とし込む。

## カテゴリ別評価

| カテゴリ | 適用可否 | 理由 |
|---|---|---|
| Resilience Patterns | N/A | Resiliency Baseline無効（`nfr-requirements.md`） |
| Scalability Patterns | N/A | 単一プロセスのローカルCLIツール（`nfr-requirements.md`） |
| Security Patterns | N/A | Security Baseline無効（`nfr-requirements.md`） |
| Performance Patterns | 適用あり（既存BRの再確認のみ） | 出力バッファリング（BR-5.3/6.1）は`business-rules.md`で決定済み。新規パターンなし |
| Logical Components | 適用あり | Question 1参照（テスタビリティを踏まえたエラー処理の実装位置） |

## Plan Checklist

- [x] Step 1: NFR Requirements成果物の分析（本ファイル作成）
- [x] Step 2-4: 未決定論点の洗い出し・質問提示（本ファイル）
- [x] Step 5: ユーザー回答収集・曖昧さ分析（Q1=B、推奨通り。曖昧・矛盾なし）
- [x] Step 6: NFR設計成果物生成（nfr-design-patterns.md, logical-components.md）
- [ ] Step 7-9: 完了メッセージ提示・承認待ち・記録

## 決定が必要な論点（質問）

### Question 1: エラー処理・終了コード変換の実装位置

`component-methods.md`は`CliRunner::run(argv: &[String]) -> ExitCode`という公開シグネチャを定義しているが、`ExitCode`を返す関数は`assert_eq!`等でのユニットテストがしづらい（`ExitCode`は内部状態を比較可能な形で公開していない）。

A) `run(argv) -> ExitCode`内部で全てのロジック（引数解析〜出力）とエラーハンドリング（stderr出力・ExitCode変換）を完結させる。`main.rs`は`CliRunner::run`を呼び出すだけの薄いラッパー

B) 中核ロジックを`run_inner(argv) -> Result<(), CliError>`のような内部関数として実装し、`CliError`を返すようにする。`component-methods.md`が要求する公開シグネチャ`run(argv) -> ExitCode`は、`run_inner`を呼び出しstderr出力・`ExitCode`変換を行う薄いラッパーとして別途用意する

C) Other（[Answer]: タグの下に詳細を記載）

[Answer]: B

**理由**: `ExitCode`型は内部状態を比較できる形で公開されておらず、`run(argv) -> ExitCode`単体では「どの引数の組み合わせでどのエラーが発生したか」をユニットテストで直接検証できない。中核ロジックを`Result<(), CliError>`を返す内部関数として実装しておけば、`assert!(matches!(result, Err(CliError::Args(...))))`のような形でユニットテスト・プロパティベーステスト双方から検証しやすくなる。公開シグネチャ`run(argv) -> ExitCode`自体は変更せず、その内部で`Result`ベースの関数を呼び出し、stderr出力とExitCode変換を行う薄いラッパーとして維持するため、Application Design時点の設計とも矛盾しない。
