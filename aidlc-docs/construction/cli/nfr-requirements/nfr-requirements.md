# NFR Requirements — cli

`cli-nfr-requirements-plan.md`のカテゴリ別評価とQ1〜Q4の決定に基づく、cliユニットの非機能要件。

## Scalability

N/A — 単一プロセスのローカルCLIツールであり、水平/垂直スケーリングの概念が適用されない。

## Availability / Resilience

N/A — Resiliency Baseline拡張機能が無効（`requirements.md` NFR-5）。単一プロセス・単一実行のCLIツールであり、リトライ・サーキットブレーカー等の耐障害性パターンは適用対象がない。

## Security

N/A — Security Baseline拡張機能が無効（`requirements.md` NFR-4）。

## Performance

- リリースビルドの最適化設定（LTO、strip、opt-level等）は追加しない。Cargoのデフォルトrelease設定に従う（Q3=A）
- 出力は全テンプレートのレンダリング成功後に一括でメモリ上のバッファから書き出す（`business-rules.md` BR-5.3/BR-6.1で既に決定済み）。ストリーミング出力は行わない（core-engineのNFR Requirements Q1=Aとの一貫性、YAGNI）

## Reliability

- エラー処理・終了コードはFunctional Design（`business-rules.md` BR-7.1〜BR-7.3）で決定済み: 成功0/失敗1のシンプルな終了コード、`mustache: <message>`形式のstderr出力
- いずれか1テンプレートの処理失敗時は全体を中断し部分出力を行わない（BR-5.3、既存方針を踏襲）

## Maintainability

- cliバイナリには`#![deny(missing_docs)]`を適用しない。公開APIを持たないバイナリであり、rustdocによる利用者向けドキュメントの必要性がcore-engineほど高くない（Q1=A）
- 可読性のための最小限のコメント（複雑な分岐の意図等）は付与する
- `CliArgs`は`clap`のderive API（`#[derive(Parser)]`）を用いて定義する。構造体定義と引数定義を一体化し、フィールド追加・変更時の同期漏れを防ぐ（Q2=A）

## Usability

- CLIの入出力インターフェース・オプション体系はFunctional Design（`business-rules.md` BR-1.x〜BR-6.x）で決定済み

## PBT運用方針（NFR-3関連）

- `proptest`を再利用する（core-engineで既に`[dev-dependencies]`へ追加済み、単一パッケージ構成のためcliのテストからもそのまま利用可能。Q4=A）
- `business-logic-model.md`（cli）のTestable Properties（DataLoaderのJSON/YAML往復変換・形式判定の決定性）はいずれもファイルI/Oを伴わない軽量な変換のため、デフォルト256ケースとする
