# NFR Requirements Plan — core-engine

`business-logic-model.md`（パース/レンダリングアルゴリズム、PBT-01 Testable Properties）と`requirements.md`のNFR-1〜NFR-6を踏まえ、core-engineユニットの非機能要件を確定する。

## Plan Checklist

- [x] Step 1: Functional Design成果物の分析
- [x] Step 2-4: 計画作成・質問洗い出し（本ファイル）
- [x] Step 5: ユーザー回答収集・曖昧さ分析（Q1=A, Q2=B, Q3=B, Q4=B、いずれも推奨通り。曖昧・矛盾なし）
- [ ] Step 6: NFR成果物生成
  - [ ] `nfr-requirements.md`
  - [ ] `tech-stack-decisions.md`
- [ ] Step 7-9: 完了メッセージ提示・承認待ち・記録

## 前提（requirements.mdより引き継ぎ、質問不要）

- Security Baseline: 無効（NFR-4）
- Resiliency Baseline: 無効（NFR-5、分散システム向け耐障害性設計は対象外）
- 配布形態: シングルバイナリでのクロスプラットフォーム配布（NFR-1）
- テスト方針: 公式specコンフォーマンススイート + PBTフル適用（NFR-2, NFR-3）
- コード品質: 著作権・ライセンスヘッダー（NFR-6、Code Generationで適用）

## 決定が必要な論点（質問）

### Question 1: レンダリング結果の出力方式
`component-methods.md`では`render(&self, template: &Template, data: &Value) -> Result<String, RenderError>`とString全体を返す方式が定義済み。非常に大きなテンプレート・データを扱う場合のメモリ効率について、追加のAPIが必要か。

A) String返却のみで十分とする。Mustacheテンプレートは通常、設定ファイルやメール本文等の小〜中規模なテキスト生成用途が主であり、ストリーミング出力（`io::Write`への逐次書き込み）を追加するコストに見合わない

B) String返却に加え、`io::Write`に逐次書き込む`render_to_writer`相当のAPIも設計に追加し、大規模出力のメモリ効率を確保する

C) Other（[Answer]: タグの下に詳細を記載）

[Answer]: A

**理由**: Mustacheの典型的な用途（設定ファイル生成、メール本文、小〜中規模なテキスト）ではストリーミング出力の必要性が薄い。実際のニーズが顕在化していない段階でストリーミングAPIを設計に組み込むのは先取り設計（YAGNI）になる。将来大規模出力の要望が出た時点で、既存のString返却APIと共存する形で追加すればよい。

### Question 2: セクション・パーシャルのネスト深度制限
Question 4（Functional Design、Q4=B）でパーシャルの循環参照は検出済みだが、循環がない場合でも極端に深いネスト（数万階層のセクション入れ子等）はRustのコールスタックを枯渇させる可能性がある。防御的な最大ネスト深度を設けるか。

A) 制限を設けない。実用的なMustacheテンプレートでそこまで深いネストは想定されず、実装もシンプルに保てる

B) 最大ネスト深度（例: 1000階層）を設け、超過時は`RenderErrorKind`に新しいバリアントを追加してエラーを返す。悪意あるまたは誤って生成された巨大テンプレートに対する防御になる

C) Other（[Answer]: タグの下に詳細を記載）

[Answer]: B

**理由**: circular partial検出（Functional Design Q4=B）と同じ理由で、Rustはスタックオーバーフロー時にプロセスが異常終了し`Result`で捕捉できない。循環がなくても病的に深いネストで同じ問題が起きうる。カウンタを再帰呼び出しに渡すだけの軽い実装コストで、`RenderError`として安全に失敗させられる一貫した防御になる。

### Question 3: 公開APIのドキュメントコメント要件
core-engineはライブラリとして配布される（NFR-1）。公開API（`pub`項目）へのrustdocコメント付与をどの程度必須とするか。

A) 全ての`pub`項目にrustdocコメントを必須とし、`#![warn(missing_docs)]`をクレートに設定する

B) 全ての`pub`項目にrustdocコメントを必須とし、`#![deny(missing_docs)]`をクレートに設定してビルド時に強制する

C) 特に強制はせず、主要な型・メソッドにのみ任意でコメントを付与する

D) Other（[Answer]: タグの下に詳細を記載）

[Answer]: B

**理由**: core-engineは配布を前提としたライブラリ（FR-2, NFR-1）であり、公開APIはApplication Designで既に`Value`, `Template`, `Mustache`等に確定・スコープされているため、コメント漏れは単なる見落としに過ぎない。`deny`でビルド時に強制しても実装コストはほぼゼロで、ドキュメント欠落を機械的に防げる。

### Question 4: PBTフレームワークの確定とテストケース数
`requirements.md`のNFR-3ではRust向け推奨として`proptest`が挙げられている（PBT-09）。これを正式採用するか、また1プロパティあたりのデフォルト試行回数の方針。

A) `proptest`を採用し、試行回数はデフォルト設定（256ケース）のまま使用する

B) `proptest`を採用し、CI実行時間を考慮してプロパティごとに試行回数を明示的に調整する（例: 軽量なプロパティは256、パーサー等重いプロパティは64等）

C) Other（[Answer]: タグの下に詳細を記載）

[Answer]: B

**理由**: `requirements.md`のNFR-3で`proptest`は既に推奨済み。ただしParser/Rendererのプロパティ（構造化されたテンプレート・ネストしたセクションを生成するもの等）はケースごとの生成・実行コストが軽量なプロパティより高くなりやすく、デフォルト256ケース一律だとCI時間が不必要に伸びる可能性がある。PBT-08（CI統合）の実用性を保つため、重いプロパティは試行回数を絞り、軽量なプロパティは標準のままとするのが妥当。
