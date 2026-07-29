# Functional Design Plan — core-engine（Mustacheオプションモジュール フルサポート）

`mustache-optional-modules-requirements.md`（FR-1〜FR-8）と`mustache-optional-modules-execution-plan.md`の決定を踏まえ、`~lambdas`・`~inheritance`・`~dynamic-names`の3モジュールに関する詳細業務ロジックを設計する。既存の`core-engine`ユニット（`unit-of-work.md`）の境界内での拡張である。

## Plan Checklist

- [x] Step 1: ユニットコンテキスト分析（unit-of-work.md、既存domain-entities.md/business-rules.md、および実装済みソース src/value.rs, src/ast.rs, src/error.rs, src/partial.rs, src/renderer.rs との整合確認）
- [x] Step 2-4: 計画作成・質問洗い出し（本ファイル）
- [ ] Step 5: ユーザー回答収集・曖昧さ分析
- [ ] Step 6: 機能設計成果物生成（既存`business-logic-model.md`/`business-rules.md`/`domain-entities.md`への追記）
- [ ] Step 7-9: 完了メッセージ提示・承認待ち・記録

## 前提（要件で既に決定済み、質問不要）

- 実装対象は3モジュールすべて（`mustache-optional-modules-requirements.md` FR-1）
- ラムダはライブラリAPI限定（FR-2）、公式spec fixture `~lambdas.json`に100%準拠（FR-3）
- テンプレート継承の親テンプレート解決は既存`PartialResolver`を流用（FR-5）
- 動的パーシャル名の値が非文字列/未定義の場合は既存の未解決パーシャルと同じ扱い（FR-7）
- ラムダの再帰レンダリングは既存`MAX_NESTING_DEPTH`ガード（`enter_depth`、`src/renderer.rs:327`）を必ず経由させる（Workflow Planningレビューで確定）

## 決定が必要な論点（質問）

### Question 1: ラムダのRust関数シグネチャ
公式spec上、ラムダはインターポレーション文脈（`{{lambda}}`、引数なしで呼ばれる）とセクション文脈（`{{#lambda}}...{{/lambda}}`、セクション本体の生テキストを引数に呼ばれる）の両方で使われうる。Rustの静的な型システム上、単一の関数シグネチャに統一する必要がある。

A) `Fn(&str) -> String`に統一する。セクション文脈ではセクション本体の生テキストを渡し、インターポレーション文脈（本体を持たない）では空文字列`""`を渡す

B) インターポレーション用・セクション用に別々の`Value`バリアント（`Value::Lambda`と`Value::SectionLambda`）を用意し、利用者に用途ごとに使い分けさせる

C) Other（[Answer]: タグの下に詳細を記載）

[Answer]: 

### Question 2: ラムダ返り値の再パース・再レンダリング方針
公式spec（`~lambdas.json`）は、ラムダの返り値を単なる文字列としてではなく、Mustacheテンプレートとして再解釈すること（返り値内の`{{var}}`等が現在のコンテキストで展開されること）を要求している。

A) 常に再パース・再レンダリングする（公式spec 100%準拠、FR-3の決定と整合）

B) 再パースは行わず、返り値の文字列をそのまま出力する（シンプルだが公式spec非準拠になる）

C) Other（[Answer]: タグの下に詳細を記載）

[Answer]: 

### Question 3: `{{lambda}}`のエスケープ規則の適用範囲
ラムダの返り値を再レンダリングした最終的な文字列に対して、通常の変数展開と同じエスケープ規則（BR-1.1/BR-1.2）を適用するか。

A) 通常の変数展開と同じ規則を適用する。`{{lambda}}`（二重波括弧）は再レンダリング後の文字列をHTMLエスケープし、`{{{lambda}}}`/`{{&lambda}}`はエスケープしない

B) ラムダの返り値は常にエスケープしない（生成元がテンプレート作成者の意図した文字列だとみなす）

C) Other（[Answer]: タグの下に詳細を記載）

[Answer]: 

### Question 4: `Value::Lambda`のPartialEq実装方針
`Value`は現在`#[derive(Debug, Clone, PartialEq)]`だが、クロージャは`PartialEq`を実装できないため手動実装が必要（Workflow Planningレビューで確認済み）。

A) `Lambda`バリアント同士の比較は常に`false`とする（自分自身との比較を含め、常に非等価）。他のバリアントとの比較は通常通り`false`

B) `Rc`のポインタ同一性（`Rc::ptr_eq`）で比較する（同一のラムダインスタンスを指していれば等価）

C) Other（[Answer]: タグの下に詳細を記載）

[Answer]: 

### Question 5: `Value::Lambda`のClone実装方針（スレッド安全性）
クロージャを保持するスマートポインタの型と、`Send`/`Sync`境界の要否（Workflow Planningレビューで確認済み）。

A) `Rc<dyn Fn(&str) -> String>`を採用する（`Send`/`Sync`不要。現在のレンダリング処理は単一スレッド同期実行のため、ライブラリ利用者に余計な境界を要求しない）

B) `Arc<dyn Fn(&str) -> String + Send + Sync>`を採用する（マルチスレッド利用を見越すが、利用者が渡すクロージャに`Send`/`Sync`実装を要求する制約が生じる）

C) Other（[Answer]: タグの下に詳細を記載）

[Answer]: 

### Question 6: セクション本体の生テキスト取得方式
既存AST（`src/ast.rs`の`Node::Section`）はパース済みの`children: Vec<Node>`のみを保持し、元のテンプレート文字列を保持していない（Workflow Planningレビューで確認済み）。ラムダのセクション文脈呼び出し（Question 1）に生テキストが必要。

A) パース時にセクション本体の生テキスト（開始タグ直後〜終了タグ直前の元の文字列）をそのまま`Node::Section`に追加フィールドとして保持する（ロスレス、実装は単純だが`Node`のメモリ使用量が増える）

B) `children`（構文木）から元のテンプレート文字列を再構築（un-parse）する処理をレンダラーに実装する（メモリ効率は良いが、再構築ロジックが元のテキストと完全に一致することを保証する追加の実装・テストコストがかかる）

C) Other（[Answer]: タグの下に詳細を記載）

[Answer]: 

### Question 7: テンプレート継承 — 親テンプレート本体内の非`{{$block}}`コンテンツの扱い
`{{<parent}}...{{/parent}}`タグの本体（子テンプレート側）に、`{{$block}}...{{/block}}`以外のテキストや他のタグが含まれていた場合の扱い。

A) 無視する（`{{$block}}`タグのみを走査してブロック名と差し替え内容を収集し、それ以外の内容は出力に一切寄与しない。公式spec準拠）

B) エラーとする（`{{<parent}}`の本体には`{{$block}}`タグ以外を許容しない）

C) Other（[Answer]: タグの下に詳細を記載）

[Answer]: 

### Question 8: テンプレート継承 — `{{<parent}}`の外で単独使用された`{{$block}}`の挙動
`{{$block}}デフォルト内容{{/block}}`が、`{{<parent}}...{{/parent}}`の外（つまり継承関係なし）で単独使用された場合。

A) 単なるデフォルト内容の表示として扱う（`{{$block}}`はデフォルト値を持つ「差し替え可能な領域」を表すタグであり、差し替えられなければデフォルト内容をそのまま表示するという意味論を継承コンテキスト外でも一貫させる）

B) エラーとする（`{{$block}}`は`{{<parent}}`の内側でのみ有効な構文とする）

C) Other（[Answer]: タグの下に詳細を記載）

[Answer]: 

### Question 9: 動的名前解決の適用範囲
`mustache-optional-modules-requirements.md`のFR-6は動的パーシャル名（`{{>* partialNameVar}}`）のみを対象としている。公式spec`~dynamic-names.json`が、パーシャルタグだけでなく継承の親タグ（`{{<parent}}`）の動的名前解決（`{{<* parentNameVar}}`）も対象に含んでいる可能性がある。

A) 要件通りパーシャルタグのみを対象とする。継承の親タグは静的な名前指定のみサポートする（公式fixtureに親タグの動的名前ケースが含まれていた場合、その部分のみ非準拠として明示的にスコープ外とする）

B) パーシャルタグに加え、継承の親タグの動的名前解決も同じ仕組み（コンテキストから文字列を解決して名前解決に渡す）で実装し、フルサポートの範囲を広げる

C) Other（[Answer]: タグの下に詳細を記載）

[Answer]: 
