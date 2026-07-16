# NFR Requirements — core-engine

`core-engine-nfr-requirements-plan.md`のQ1〜Q4の決定と、`requirements.md`のNFR-1〜NFR-6を統合した、core-engineユニットの非機能要件。

## スケーラビリティ
**N/A**。core-engineは単一プロセス内で完結するライブラリであり、水平/垂直スケーリングの概念が適用されない（Resiliency Baseline無効、NFR-5）。

## パフォーマンス
- レンダリング結果は`String`として一括生成する（Q1=A）。ストリーミング出力（`io::Write`逐次書き込み）は現時点のスコープ外とし、将来必要になった場合は既存APIと共存する形で追加を検討する
- 特定のレイテンシ・スループット数値目標は設けない（CLIツールとしての対話的用途が主であり、要件定義（`requirements.md`）にも具体的な性能目標の記載はない）

## 可用性
**N/A**。単一プロセスのライブラリ/CLIであり、稼働率・障害復旧（DR）・フェイルオーバーの概念が適用されない（NFR-5）。

## セキュリティ
**N/A（無効）**。Security Baseline拡張機能は要件定義時にオプトアウト済み（NFR-4）。ローカルCLIツールであり、テンプレート・データとも利用者自身が用意する想定のため、本プロジェクトの現段階ではセキュリティベースラインを強制しない。

## 信頼性
- パーシャルの循環参照は検出し、`RenderErrorKind::PartialCycleDetected`として明示的にエラーを返す（Functional Design Q4=B）
- セクション・パーシャルの再帰ネストに最大深度制限を設け、超過時は新しい`RenderErrorKind`バリアントでエラーを返す。スタックオーバーフローによるプロセスの異常終了を防ぐ（Q2=B）
- Parser/Rendererのエラーは全て`Result`型で呼び出し元に伝播し、`panic`に頼らない（`business-logic-model.md`のエラー伝播方針）

## 保守性
- 全ての公開API（`pub`項目）にrustdocコメントを必須とし、`#![deny(missing_docs)]`をクレートルート（`lib.rs`）に設定してビルド時に強制する（Q3=B）
- 全ソースファイルに著作権・ライセンスヘッダーを付与する（NFR-6、Code Generationで適用）
- テストは公式mustache/specコンフォーマンススイート（例示ベース）と`proptest`によるプロパティベーステスト（PBT-01で識別済みのプロパティ）を組み合わせる（NFR-2, NFR-3、Q4=B）

## ユーザビリティ（ライブラリ利用者体験）
- `ParseError`/`RenderError`は発生箇所の行番号・列番号を含み、テンプレート作成者のデバッグを支援する（Functional Design Q5=B）
- 公開APIのドキュメントコメント（Q3=B）により、ライブラリ利用者が`cargo doc`で仕様を参照できる

## 技術スタック選定
詳細は[tech-stack-decisions.md](tech-stack-decisions.md)を参照。
