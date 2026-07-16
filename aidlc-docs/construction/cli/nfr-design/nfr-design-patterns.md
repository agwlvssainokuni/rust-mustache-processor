# NFR Design Patterns — cli

## N/A カテゴリ

| カテゴリ | 理由 |
|---|---|
| Resilience Patterns | Resiliency Baseline無効。単一プロセス・単一実行のCLIツールであり、リトライ・サーキットブレーカー等の適用対象がない |
| Scalability Patterns | 単一プロセスのローカルCLIツールであり、水平/垂直スケーリングの概念が適用されない |
| Security Patterns | Security Baseline無効 |

## パターン1: Result-based Core Logic（Question 1）

**目的**: `component-methods.md`が要求する公開シグネチャ`CliRunner::run(argv) -> ExitCode`を維持しつつ、テスタブルな内部構造にする。

**実装方針**: 中核ロジックを`run_inner(argv: &[String]) -> Result<(), CliError>`として実装する。公開関数`run(argv) -> ExitCode`は`run_inner`を呼び出し、`Err`の場合はBR-7.2形式でstderrへ出力して`ExitCode::from(1)`を、`Ok`の場合は`ExitCode::from(0)`を返す薄いラッパーとする。これによりユニットテスト・プロパティベーステストは`run_inner`を直接呼び出し、`Result`の中身（`CliError`のバリアント）を検証できる。

## パターン2: Atomic Output Buffering（BR-5.3, BR-6.1の実装パターン化）

**目的**: 複数テンプレートのいずれか1つでもエラーが発生した場合に、部分的な出力を一切書き出さないことを保証する。

**実装方針**: 全テンプレートのレンダリング結果を`String`バッファに追記していき、ループの途中でエラーが発生した場合は即座に`Err`を返してバッファを破棄する（`write_output`を呼び出さない）。全テンプレートの処理が成功した場合のみ、最後に1回だけ`IoController::write_output`を呼び出す。core-engineのCapacity Pre-allocationパターン（NFR Design Q3）と同様、事前に各テンプレートの`content.len()`合計程度の容量を`String::with_capacity`で確保してもよいが、必須ではない（テンプレート数・サイズは実行前に読み込み済みのため容易に見積もれる）。

## パターン3: Per-Template Partial Resolver Construction（BR-4.1〜4.3の実装パターン化）

**目的**: `--partials-dir`未指定時、テンプレートファイルごとに異なるデフォルトパーシャルディレクトリを解決する（BR-4.2）。

**実装方針**: `Mustache`はビルダースタイルで`with_partial_resolver`が`self`を消費するため、テンプレートごとに異なるパーシャルディレクトリを使う場合、テンプレートごとに新しい`Mustache`インスタンスを構築する（`Mustache::new().with_strict(args.strict).with_partial_resolver(Box::new(DirectoryPartialResolver::new(dir)))`）。`strict`設定は全テンプレート共通のためループ内で毎回同じ値を渡す。`Mustache`・`DirectoryPartialResolver`はいずれも軽量な値オブジェクトであり、テンプレートごとの再構築によるパフォーマンス上の懸念はない（実行時間の大半はファイルI/Oとレンダリングそのものが占める）。

## パターン4: Unified CliError with Display（BR-7.2, BR-7.3の実装パターン化）

**目的**: `CliArgsError`/`IoError`/`DataLoaderError`/`mustache_processor::error::Error`という異種のエラー型を、`?`演算子で一箇所に集約し、統一されたメッセージ形式でstderrへ出力する。

**実装方針**: `domain-entities.md`で定義済みの`CliError`列挙型に対し、各エラー型からの`From`実装（`impl From<CliArgsError> for CliError`等）を用意し、`run_inner`内で`?`演算子による自動変換を可能にする。`CliError`に`std::fmt::Display`を実装し、`mustache: {error}`（BR-7.2）の`{error}`部分に相当するメッセージを種別ごとに整形する。`CliError::Mustache(e)`の場合は`e`（`mustache_processor::error::Error`）が既に行番号・列番号を含む`Display`実装を持つため、そのまま委譲する。
