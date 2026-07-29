# Functional Design Plan — core-engine: BR-10.7 ブロック再インデント処理

## 対象

`~inheritance`の既知の制限4ケース（`inheritance_known_limitations`、`#[ignore]`属性）を解消する。

- Standalone block
- Block reindentation
- Intrinsic indentation
- Nested block reindentation

## 前提となる仕様調査結果（要件確認済み）

公式`mustache/spec`リポジトリのIssue #130（PR #131の設計議論）から、以下のアルゴリズムを取得し、3/4ケースをバイト単位で手動検証済み（詳細は`audit.md`参照）。

- **Rule 1（標準スタンドアロン判定）**: 開始タグが行頭clear かつ 終了タグが行末clearなら、そのペアはスタンドアロン（`BB`/`BE`/`SB`/`SE`）。ただし**引数ブロック（オーバーライド側）はこの規則の対象外**（パラメータブロック＝展開先のみ対象）。
- **Rule 2（intrinsic indentation）**: 開始タグが行末clearなら、開始タグ直後の最初の行の先頭空白が「本来のインデント」。引数ブロックは`B`/`E`系全般、パラメータブロックは`BB`/`BE`のみ対象。
- **Rule 3（定義箇所での除去）**: 本来のインデントをブロック内の全行から除去する。
- **Rule 4（展開箇所での付与）**: ①intrinsic indentationがあればそれを使用、②なければスタンドアロンペアの開始タグのインデントを使用、③どちらもなければ空文字列。
- **追加で発見した細則**: 終了タグが行頭または行末のいずれかでclearする場合、除去後の値に末尾改行を強制的に付与する（"Standalone block"ケースの検証で判明）。

「Nested block reindentation」（3階層継承）は複雑なため手動検証は未完了だが、後述の実装方針（生テキストへのインデント事前適用＋再パース）を採ることで、既存のBR-5.4/BR-10.6と同じ仕組みが自然にカスケードし、追加の特別処理なしに解決する見込み。

## Functional Design ステップ

- [ ] Step 1: 質問への回答を収集・分析
- [ ] Step 2: `business-rules.md`のBR-10.7を確定アルゴリズムで書き換え
- [ ] Step 3: `domain-entities.md`の`Node::Block`定義を更新（新規フィールド）
- [ ] Step 4: `business-logic-model.md`にブロック再インデントの処理フローとTestable Propertiesを追記
- [ ] Step 5: 完了報告・承認待ち

## 質問

### Question 1: 実装方式

ブロックの再インデントを実現する実装方式は、既存のパーシャル／親テンプレートのインデント処理（BR-5.4/BR-10.6、「値展開前の生テキストにインデントを適用してから再パースする」方式）と同じアプローチを踏襲するのが良いと考えます。

A) 生テキスト方式（推奨） — オーバーライドの生テキスト（`Node::Block.raw`相当を新設）に対し、Rule3の除去とRule4の付与を文字列レベルで適用してから`crate::parser::parse`で再パースする。既存のPartial/Parent/ラムダのセクション文脈再パースと同じパターンを再利用でき、多段ネスト（Nested block reindentation）も特別な追加処理なしに自然にカスケードする

B) AST方式 — `Node`木を直接走査し、`Text`ノードの先頭に再帰的にインデント文字列を挿入する専用ロジックを新設する

C) レンダリング後方式 — ブロックの子要素を通常通りレンダリングした後、出力文字列に対して事後的にインデントを適用する（BR-5.4で「値展開後のインデント事後適用は改行を誤って波及させる」問題が既に判明しているため非推奨）

D) Other (please describe after [Answer]: tag below)

[Answer]: A

### Question 2: clearance（スタンドアロン判定・intrinsic indentation）情報の計算タイミング

Rule 1/2で必要な「タグが行頭／行末でclearするか」の判定と、intrinsic indentation値は、どの段階で計算するのが良いですか。

A) パース時（推奨） — 既存の`apply_standalone_trimming`（Partial/Parent/Sectionの`indent`フィールドを計算している箇所）を拡張し、`Node::Block`のパース時に一度だけ計算してASTフィールドとして保持する。レンダリング時は保持済みの値を使うだけで済み、既存の設計パターン（BR-5.4/BR-10.6/BR-9.3の`open`/`close`保持）と一貫する

B) レンダリング時 — `Node::Block`は生テキストのみ保持し、レンダリング時に毎回clearance判定・intrinsic indentation計算をやり直す

C) Other (please describe after [Answer]: tag below)

[Answer]: A

### Question 3: 「Nested block reindentation」の検証方針

4ケース目（3階層継承、`{{<grandparent}}`→`{{<parent}}`→トップレベル）は手動でのバイト単位検証が完了していません。

A) 実装方針（Question 1で生テキスト＋再パース方式を採用）に基づき実装し、`cargo test --test spec -- --ignored inheritance_known_limitations`で実地検証、不一致があれば都度アルゴリズムを補正する（推奨・本プロジェクトで確立された「実測に基づく検証」の方針に合致）

B) コードを書く前に、Case Dも含めた4ケース全てを手動でバイト単位トレースし切ってから実装に着手する（時間がかかるが着手前の確信度は最大）

C) Other (please describe after [Answer]: tag below)

[Answer]: A

### Question 4: テストの統合方針

現在`inheritance_known_limitations`として`#[ignore]`分離されている4ケースは、実装完了・全件合格が確認できた場合、どう扱いますか。

A) `inheritance()`本体に統合し、`#[ignore]`属性と`inheritance_known_limitations`関数自体を削除する（推奨 — 既知の制限でなくなった以上、分離しておく理由がなくなるため。`~inheritance`が27/27になったことをドキュメントにも反映する）

B) 回帰検知のため`inheritance_known_limitations`関数は残しつつ`#[ignore]`のみ外す

C) Other (please describe after [Answer]: tag below)

[Answer]: A

### Question 5: リリース種別

このFunctional Designの完了後、Code Generationを経てリリースする場合のバージョン種別はどれが適切ですか。新規公開APIの追加はなく、既存の`~inheritance`サポートの正確性向上（23/27→27/27）のみです。

A) パッチリリース（v0.2.1） — 後方互換な不具合修正のため（推奨、semver準拠）

B) マイナーリリース（v0.3.0）

C) Other (please describe after [Answer]: tag below)

[Answer]: A

### Question 6: 既知の制限に関するドキュメント記述

`business-rules.md`のBR-10.7、`README.md`/`README.en.md`、`mustache-optional-modules-requirements.md`に記載済みの「既知の制限」の記述は、実装完了後どう扱いますか。

A) 全て書き換えて「既知の制限」の記述を削除し、spec準拠率を27/27（100%）に更新する（推奨 — 実態を反映した記録を保つという本プロジェクトの一貫した方針）

B) 「既知の制限」の記述を残したまま、末尾に修正済みである旨を追記する（履歴として保持）

C) Other (please describe after [Answer]: tag below)

[Answer]: A
