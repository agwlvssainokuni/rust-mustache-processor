# Code Generation Plan — core-engine（Mustacheオプションモジュール フルサポート）

`mustache-optional-modules-requirements.md`・`mustache-optional-modules-execution-plan.md`・Functional Design成果物（`business-logic-model.md`/`business-rules.md`/`domain-entities.md`のBR-9〜BR-11、v0.2.0追記分）に基づき、既存`core-engine`ユニットのソースを修正・拡張する。ブラウンフィールド（既存ファイルの修正）であり、`ClassName_new.rs`等の重複ファイルは作成しない。

- **ワークスペースルート**: `~/Documents/project/git/rust-mustache-processor`
- **対象ユニット**: core-engine（`src/lib.rs`, `src/value.rs`, `src/ast.rs`, `src/parser.rs`, `src/renderer.rs`, `src/partial.rs`, `src/error.rs`）
- **本計画がCode Generationの単一のソースオブトゥルース**。各Step完了ごとにチェックボックスを更新し、コミットする（既存プロジェクトの慣行を踏襲）

## Plan Checklist

- [x] Step 1: 公式spec fixture取得（`~lambdas.json`, `~inheritance.json`, `~dynamic-names.json`をmustache/specリポジトリから取得し`tests/spec/fixtures/`に配置）
- [x] Step 2: `Value::Lambda`実装（`src/value.rs`）
- [x] Step 3: AST拡張（`src/ast.rs`）
- [x] Step 4: パーサー拡張（`src/parser.rs`）
- [x] Step 5: レンダラー拡張 — ラムダ（`src/renderer.rs`）
- [x] Step 6: レンダラー拡張 — テンプレート継承（`src/renderer.rs`）
- [x] Step 7: レンダラー拡張 — 動的パーシャル名（`src/renderer.rs`）
- [x] Step 8: spec conformanceテスト統合・是正ループ（`tests/spec/`） — `~lambdas`10/10・`~dynamic-names`21/21全件成功。`~inheritance`は23/27（85%）。残り4件（ブロック再インデント処理）は既知の制限としてBR-10.7に記録し、フォローアップ課題とする（ユーザー承認済み）
- [x] Step 9: ユニットテスト追加（`src/value.rs`, `src/renderer.rs`等の`#[cfg(test)]`） — 12件追加、既存72件と合わせ計84件全成功
- [ ] Step 10: プロパティベーステスト追加（`tests/proptest/`）
- [ ] Step 11: バージョン更新（`Cargo.toml` 0.1.1 → 0.2.0）
- [ ] Step 12: ドキュメント更新（`README.md`/`README.en.md`）
- [ ] Step 13: Code Generation Summary作成（`aidlc-docs/construction/core-engine/code/summary.md`への追記）

## Step詳細

### Step 1: 公式spec fixture取得
- [x] `https://github.com/mustache/spec`の`specs/`配下から`~lambdas.json`・`~inheritance.json`・`~dynamic-names.json`を取得し、既存の6ファイルと同じ形式で`tests/spec/fixtures/`に配置した
- [x] 取得したfixtureの内容を精査し、Functional Design（BR-9〜BR-11）との齟齬を発見・是正した（Step 8を待たずこの時点で修正。齟齬が軽微な場合はStep 8のspec conformanceループで是正する方針は継続）:
  - **ラムダの再パースデリミタが文脈依存**（BR-9.3を修正）: インターポレーション文脈は常にデフォルトデリミタ、セクション文脈はそのタグ自身が有効だった時点のデリミタを使う。`Node::Section`に`open`/`close`フィールドを追加（domain-entities.md反映済み）
  - **ラムダはキャッシュしない**（BR-9.3b追加）: 同一ラムダへの複数回参照はその都度呼び出す
  - `~dynamic-names.json`のテストケースを確認した結果、動的名前解決はパーシャルタグのみが対象で親タグの動的名前ケースは含まれていないことを確認した（Functional Design Q9=Aの決定が公式spec通りであることの裏付け、修正不要）

### Step 2: `Value::Lambda`実装
- `src/value.rs`の`Value`列挙型に`Lambda(Rc<dyn Fn(&str) -> String>)`を追加
- `Value`の`#[derive(Debug, Clone, PartialEq)]`を手動実装に置き換える（`Lambda`以外は既存のderive相当の挙動を保つ。`Lambda`の`Debug`は固定文字列、`Clone`は`Rc`複製、`PartialEq`は常に`false`、BR-9.6/BR-9.7）
- `is_truthy`に`Value::Lambda => true`を追加（BR-9.5、常にtruthy）
- ライブラリ利用者向けのコンストラクタ手段（`Value::Lambda(Rc::new(...))`を直接構築可能にする、または補助関数を用意するかは実装時に決定）

### Step 3: AST拡張
- `src/ast.rs`の`Node::Section`に`raw: String`（BR-9.2、セクション本体の生テキスト）と`open: String`/`close: String`（BR-9.3、パース時点で有効だったデリミタ）を追加
- `PartialName`列挙型（`Static(String)` / `Dynamic(String)`）を追加し、`Node::Partial.name`の型を`String`から`PartialName`に変更（BR-11.1）
- `Node::Parent { name: String, children: Vec<Node>, indent: String, pos: SourcePosition }`を追加（BR-10.1/BR-10.2/BR-10.6）
- `Node::Block { name: String, children: Vec<Node>, pos: SourcePosition }`を追加（BR-10.3/BR-10.4）

### Step 4: パーサー拡張
- Pass 1（tokenize）に新規タグ種別判定を追加: 親タグ（`<`）、ブロックタグ（`$`）、動的パーシャル（`>`直後の`*`）
- Pass 2（スタンドアロン判定）のブロックタグ種別に、親タグ・終了タグ・ブロックタグ・終了タグ・動的パーシャルタグを追加する（BR-7.5）
- Pass 3（木構築）に、親タグ・ブロックタグのスタックベース対応付け（開始〜終了のマッチング、`UnbalancedSection`/`UnexpectedEof`エラーの流用）と、セクションの`raw`/`open`/`close`記録、動的パーシャル名（`*`接頭辞）の`PartialName::Dynamic`判定を追加

### Step 5: レンダラー拡張 — ラムダ
- `Node::Variable`/`Node::Section`の名前解決結果が`Value::Lambda`の場合の分岐を追加（BR-9.1）
- 呼び出し（`Fn(&str) -> String`、セクションは`raw`、変数展開は`""`、BR-9.2）を実装
- 返り値の再パース・再レンダリングを実装（BR-9.3）: インターポレーション文脈は常にデフォルトデリミタ（`{{`/`}}`）、セクション文脈は`Node::Section.open`/`.close`で再パースする。いずれも現在のコンテキストスタックを使い、`enter_depth`によるネスト深度ガードを経由する。ラムダは参照の都度呼び出し、結果をキャッシュしない（BR-9.3b）
- `{{lambda}}`/`{{{lambda}}}`/`{{&lambda}}`のエスケープ規則の適用（BR-9.4）
- 逆セクション文脈でラムダを常にtruthyとして扱う（BR-9.5、呼び出しなし）

### Step 6: レンダラー拡張 — テンプレート継承
- `Node::Parent`到達時の処理: `PartialResolver::resolve`呼び出し（BR-10.1）、`indent`適用（BR-10.6）、自身の`children`からオーバーライドマップ構築（BR-10.2）、親テンプレートのパース、オーバーライドを適用したレンダリング（BR-10.3）
- `Node::Block`到達時（`Node::Parent`のオーバーライド解決を経由せず、通常の子ノードとして走査された場合）、自身の`children`をそのままレンダリング（BR-10.4）
- 多段継承時のオーバーライド伝播は、Step 1で取得した`~inheritance.json`の実際のテストケースを見て実装方針を確定する（Functional DesignのBR-10.5で明示的に先送りされた論点）

### Step 7: レンダラー拡張 — 動的パーシャル名
- `Node::Partial`の`name`が`PartialName::Dynamic(var)`の場合、`var`をコンテキスト探索で解決し（BR-11.1）、`Value::String`であれば通常のパーシャル解決処理へ、そうでなければBR-5.2相当の未解決パーシャル処理へ分岐（BR-11.2）
- `PartialName::Static(name)`の場合は既存のパーシャル処理をそのまま適用

### Step 8: spec conformanceテスト統合・是正ループ
- `tests/spec/conformance.rs`（既存の必須6モジュールを検証している統合テスト）に、Step 1で取得した3fixtureを対象としたテストケースを追加する
- `cargo test --test spec`を実行し、失敗したケースを1つずつ調査する。Functional Design（BR-9〜BR-11）と実際のfixture期待値に齟齬があれば、業務ルール文書（`business-rules.md`）を修正した上で実装を是正する（既存必須6モジュールのStep8と同じ「実測に基づき設計を是正する」方針）
- 3モジュール合計のテストケースが100%成功するまで繰り返す

### Step 9: ユニットテスト追加
- `src/value.rs`: `Value::Lambda`の`is_truthy`/`Debug`/`Clone`/`PartialEq`（常にfalse）の単体テスト
- `src/renderer.rs`: ラムダの再帰レンダリングが`MAX_NESTING_DEPTH`ガードで終端することの単体テスト（自己参照するラムダの明示的なテストケース）
- `src/renderer.rs`: テンプレート継承のオーバーライド解決・デフォルト内容表示の単体テスト
- `src/renderer.rs`: 動的パーシャル名の解決・未解決時の単体テスト

### Step 10: プロパティベーステスト追加
- `tests/proptest/`に、`business-logic-model.md`のTestable Properties（v0.2.0追加分）を実装:
  - ラムダの再帰レンダリングがネスト深度ガードで終端する（Invariant）
  - オーバーライドされないブロックはデフォルト内容と一致する（Invariant）

### Step 11: バージョン更新
- `Cargo.toml`の`version`を`0.1.1`から`0.2.0`に変更（NFR-2、マイナーバージョンアップ）

### Step 12: ドキュメント更新
- `README.md`/`README.en.md`: 対応機能一覧に3モジュールを追加し、「対応していない機能」の記述を更新
- ライブラリ利用者向けにラムダAPIの使用例を追記（`Value::Lambda`の構築方法）

### Step 13: Code Generation Summary作成
- `aidlc-docs/construction/core-engine/code/summary.md`に、本Code Generationで追加・変更したファイル一覧、テスト構成、spec準拠状況（9モジュール・ケース数）を追記する
