# Code Generation Plan — core-engine

## ユニットコンテキスト

- **要件**: FR-1, FR-2（ライブラリ本体）, FR-4, FR-6（抽象化部分）, FR-7, FR-8（`unit-of-work-story-map.md`）
- **依存**: なし（`unit-of-work-dependency.md`よりcore-engineは他ユニットに依存しない）
- **参照する承認済み成果物**:
  - `aidlc-docs/inception/application-design/components.md`, `component-methods.md`（Value/Mustache/Parser/Renderer/PartialResolver/DirectoryPartialResolverの公開インターフェース）
  - `aidlc-docs/construction/core-engine/functional-design/`（domain-entities.md, business-rules.md, business-logic-model.md）
  - `aidlc-docs/construction/core-engine/nfr-requirements/`（tech-stack-decisions.md: serde + proptest, missing_docs強制）
  - `aidlc-docs/construction/core-engine/nfr-design/`（nfr-design-patterns.md, logical-components.md: RenderState, Recursion Guard, Cycle Detection, Capacity Pre-allocation）

## 計画作成時の整合性補正（要記録）

Functional Designの`domain-entities.md`はValue型を`Integer`/`Float`/`HashMap`ベースで設計したが、これは既承認の`component-methods.md`（Application Design）で定義済みの以下の仕様を見落としていた:
- `Value::from_serialize<T: Serialize>(value: &T) -> Result<Value, ValueError>`
- `Value::is_truthy(&self) -> bool`
- `Value::get(&self, key: &str) -> Option<&Value>`
- `Value::iter(&self) -> Option<impl Iterator<Item = &Value>>`
- `components.md`: 「Map（キー順序を保持するMap）」

本計画ではApplication Design（先に承認済み・上位）を正としてこれらを取り込みつつ、Functional Designの正当な詳細化（`Number`を`Integer`/`Float`に分割する判断はApplication Designの「Number」を具体化したものであり矛盾しない）は維持する。キー順序保持は、新規依存クレート（`indexmap`等）を追加せず、`Vec<(String, Value)>`ベースの内部`Map`型で実現する（`tech-stack-decisions.md`で承認済みの依存はserde/proptestのみのため、依存追加を避ける）。

## Plan Checklist

### Step 1: Project Structure Setup（Greenfield）
- [x] `Cargo.toml`を更新: `[lib]`ターゲット追加、`serde`（`derive`機能）を`[dependencies]`に追加、`proptest`を`[dev-dependencies]`に追加
- [x] `src/lib.rs`（クレートルート、`#![deny(missing_docs)]`、モジュール宣言、著作権ヘッダー）を作成
- [x] `src/value.rs`, `src/ast.rs`, `src/parser.rs`, `src/renderer.rs`, `src/partial.rs`, `src/error.rs`の空ファイルを著作権ヘッダー付きで作成（`error.rs`・`ast.rs`はStep3内容を含めて本実装まで先行実施、`value.rs`はStep2で本実装）

### Step 2: Business Logic Generation — Value / Map
- [x] `src/value.rs`: `Value`列挙型（Null, Bool, Integer(i64), Float(f64), String(String), Array(Vec\<Value\>), Map(Map)）、`Map`型（`Vec<(String, Value)>`ベース、キー順序保持、`get`/`insert`/`iter`）
- [x] `Value::is_truthy`（BR-2.1〜BR-2.4準拠）、`Value::get`、`Value::iter`を実装
- [x] `Value::from_serialize<T: Serialize + ?Sized>`を、`serde::Serializer`をカスタム実装した内部シリアライザ経由で実装、`ValueError`型を定義
- [x] `cargo build --lib`成功を確認（`ast.rs`未使用によるdead_code警告のみ、Step3で解消予定）

**実装時の追加補正（要記録）**: Application Design（`components.md`）の真偽判定規則の要約「false, null, 空文字列, 空配列, 空Mapはfalsy」は、Functional Design（`business-rules.md` BR-2.1〜BR-2.4、公式mustache/spec準拠で精査済み）と矛盾する（公式spec上、空文字列・空Mapはtruthy）。components.mdの記述はApplication Design段階での大まかな要約であり、business-rules.mdはFunctional Designステージで公式spec準拠を目的として精査された記述のため、後者を正として`is_truthy`を実装した。

### Step 3: Business Logic Generation — AST / Parser
- [x] `src/ast.rs`: `SourcePosition`, `Node`（Text/Variable/Section/Partial）を定義（`domain-entities.md`準拠。Step1で先行作成済み）
- [x] `src/parser.rs`: タグ検出・デリミタ変更追跡・スタンドアロン行トリミング（BR-6.1〜BR-7.2）・セクション対応チェックを実装する`parse`関数
- [x] `src/error.rs`: `ParseError`, `ParseErrorKind`を定義（行番号・列番号付き、Q5=B。Step1で先行作成済み）
- [x] `parser.rs`に`#[cfg(test)]`ユニットテスト17件（Step7の一部前倒し）を実装し、`cargo test --lib parser::`で全件成功を確認

### Step 4: Business Logic Generation — Renderer
- [x] `src/renderer.rs`: `RenderState`（context_stack, depth, partial_chain, strict）、`MAX_NESTING_DEPTH`定数
- [x] 変数展開（エスケープ有無、BR-1.1〜BR-1.9）、セクション/逆セクション評価（BR-2.1〜BR-3.1）、コンテキストスタック探索（BR-4.1〜BR-4.2）を実装
- [x] パーシャル解決（BR-5.1〜BR-5.5: 遅延評価、常にエラー化、循環検出、インデント適用）とネスト深度制限（Recursion Guardパターン）を実装
- [x] `src/error.rs`に`RenderError`, `RenderErrorKind`（UndefinedVariable, PartialNotFound, PartialCycleDetected, MaxNestingDepthExceeded, **PartialParseError**）を追加（後者は実装時に追加、下記参照）
- [x] `renderer.rs`に`#[cfg(test)]`ユニットテスト21件（Step7の一部前倒し）を実装し、`cargo test --lib renderer::`で全件成功を確認

**実装時の追加補正（要記録・1）**: `domain-entities.md`/`logical-components.md`の`RenderErrorKind`にはパーシャル内容自体の構文エラーを表すバリアントが定義されていなかった（パーシャルは独立ファイルであり、主テンプレートのパース成功後でも不正なMustache構文を含みうるため、この経路のエラー化が必須）。`RenderErrorKind::PartialParseError { name, message }`を追加し、BR-8.2（パーシャル内での位置報告）に従いパーシャル自身の`ParseError`のline/columnをそのまま採用した。

**実装時の追加補正（要記録・2）**: NFR Design Q2で決定した`MAX_NESTING_DEPTH`の上限値は、NFR Requirements Q2の「例1000階層」を踏襲したものだったが、実装後にユニットテストで実測したところ、Rustのデフォルトスレッドスタック（特にWindows既定の1MiB相当を`RUST_MIN_STACK=1048576`で模擬した場合）では、深度1000はガード自体が発火する前に実スタックオーバーフローを起こすことが判明した（`RenderState`を伴う`render_nodes`/`render_section`の再帰1コールあたりのスタックフレームが、単純な再帰関数より大きいため）。二分探索的に実測した結果、1MiBスタックでも安全に動作する上限は200階層程度だったため、余裕を持たせて`MAX_NESTING_DEPTH = 100`に修正した。NFR Requirements Q2・NFR Design Q2の原文はいずれも「例」「上限値1000は例示通り」と明記しており値そのものを厳密に固定してはいなかったため、Code Generationステージでの実装知見に基づく妥当な具体化と判断した。実用上、100階層は現実的なMustacheテンプレートのネスト深度を十分にカバーする。

### Step 5: Business Logic Generation — PartialResolver / DirectoryPartialResolver
- [x] `src/partial.rs`: `PartialResolver`トレイト（`resolve(&self, name: &str) -> Option<String>`、`component-methods.md`準拠。Step4でRendererが依存するため先行作成済み）
- [x] `DirectoryPartialResolver`（ディレクトリベース実装、`{name}.mustache`ファイルを読み込み）を実装、ユニットテスト2件で動作確認

### Step 6: Business Logic Generation — Mustache エンジン公開API
- [x] `src/lib.rs`: `Template`（`root: Vec<Node>`, `source_len: usize`）、`Mustache`（`new`, `with_partial_resolver`, `with_strict`, `parse`, `render`, `render_str`）、`Error`型（Parse/Render統合）を実装
- [x] `Mustache::render`で`String::with_capacity(template.source_len)`による事前確保（Capacity Pre-allocationパターン）を適用
- [x] 全公開項目にrustdocコメントを付与（`#![deny(missing_docs)]`がビルドを通ることを確認、警告0件）
- [x] クレートレベルdocコメントに使用例（doctest）を記載し`cargo test --doc`で成功を確認（Step10の一部前倒し）
- [x] `lib.rs`に統合テスト5件（render_str/parse+render再利用/エラー伝播/PartialResolver連携）を実装し`cargo test --lib`で全45件成功を確認

**実装時の追加補正（要記録）**: `Cargo.toml`の`[lib]`セクションにライブラリ名を指定していなかったため、パッケージ名`rust-mustache-processor`由来の既定クレート名`rust_mustache_processor`になっていた。doctestで利用しやすい名前とするため`name = "mustache_processor"`を明示した（Step1のCargo.toml内容への軽微な追加補正）。

### Step 7: Business Logic Unit Testing
- [x] `value.rs`内に`#[cfg(test)]`ユニットテスト14件（is_truthy各パターン、get/iter、Map挿入順序、from_serialize: プリミティブ/struct/Vec/Map/Option/ネスト構造）
- [x] `parser.rs`内に`#[cfg(test)]`ユニットテスト17件（各タグ種別、デリミタ変更、スタンドアロントリミング、構文エラー）— Step3で前倒し実施済み
- [x] `renderer.rs`内に`#[cfg(test)]`ユニットテスト21件（エスケープ、セクション各パターン、strictモード、パーシャル未解決/循環/深度超過）— Step4で前倒し実施済み
- [x] `lib.rs`内に統合テスト5件（render_str/parse+render再利用/エラー伝播/PartialResolver連携）、`partial.rs`内にユニットテスト2件 — Step5/6で前倒し実施済み
- [x] `cargo test --lib`で全59件成功を確認

### Step 8: Spec Conformance Test Generation
- [x] 公式mustache/specリポジトリ（`https://github.com/mustache/spec`）より必須モジュール（comments, delimiters, interpolation, inverted, partials, sections）のJSON定義を取得し`tests/spec/fixtures/`に配置（計136テストケース）
- [x] `tests/spec/main.rs` + `tests/spec/conformance.rs`: フィクスチャを読み込み、各テストケースをMustacheエンジンで実行し期待出力と比較する統合テストハーネスを実装（NFR-2）。フィクスチャJSON解析用に`serde_json`を`[dev-dependencies]`に追加（テスト専用、ライブラリ本体の依存には影響しないため`tech-stack-decisions.md`の逸脱として許容）
- [x] `cargo test --test spec`で136件全て成功することを確認（複数回のイテレーションで下記の設計補正を実施）

**実装時の重大な補正（要記録）**: 実際に公式spec conformanceテストを実行したところ、当初の設計・実装には次の不備があり、いずれも修正した。詳細は`business-rules.md`・`business-logic-model.md`・`logical-components.md`にも反映済み:

1. **暗黙のイテレータ`{{.}}`が未実装だった**: 現在のコンテキスト自体を参照する機能。当初「公式spec対象外」と誤って判断していたが、`interpolation.json`/`sections.json`に明確に含まれる必須機能だった。`renderer::resolve`に実装（BR-1.10, BR-2.6）。
2. **ドット区切り名前（`{{a.b.c}}`）が未実装だった**: 同じく誤って対象外と判断していたが必須機能。最初のセグメントのみコンテキストスタック探索、以降は直接のキー参照として実装（BR-1.11）。データ中の`"a.b"`という単一フラットキーには絶対にマッチしないことを確認（"Dotted Names are never single keys"）。
3. **スタンドアロンタグ判定が1行に複数のブロックタグがある場合・`\r\n`改行に対応していなかった**: 当初はタグ単体の前後テキストのみを見る局所判定だったため、`{{#a}}{{/a}}\n`のような複数タグの行や`\r\n`改行を誤判定していた。行全体を単位とする3パス構成（tokenize → 行単位スタンドアロン判定 → 木構築）に`parser.rs`を全面的に書き直した（BR-7.2〜7.4）。
4. **スカラー真値セクションでコンテキストがプッシュされていなかった**: `{{#foo}}{{.}} is {{foo}}{{/foo}}`（foo="bar"）のようなケースで`{{.}}`が解決できなかった。Map同様にスカラー値自体もプッシュするよう修正（BR-2.4修正）。
5. **パーシャル未解決時に常時エラーとしていた**: Application Design Q3=A/Functional Design Q3=Aの決定に基づき実装していたが、公式spec（"Failed Lookup"）は空文字列を期待していた。strictモード時のみエラーとするよう修正し、FR-7のstrictモードの対象をパーシャルにも拡張した（BR-5.2修正）。
6. **パーシャル循環検出（名前チェーン追跡）が公式spec違反だった**: Functional Design Q4=Bで決定した設計だったが、"Recursion"テストにより同名パーシャルの自己再帰（データに基づき自然終端するツリー/リスト構造など）は正当な実装パターンであり、名前の再出現だけで一律にエラーとしてはならないと判明。`RenderState`から`partial_chain`フィールドと`RenderErrorKind::PartialCycleDetected`を削除し、`MAX_NESTING_DEPTH`（ネスト深度ガード）のみを安全装置とする設計に変更した（BR-5.5削除）。
7. **パーシャルのインデントをレンダリング後の出力に事後適用していた**: "Standalone Indentation"テストにより、値展開（例:`{{{content}}}`が複数行の値を挿入するケース）で生じた改行にまでインデントが波及する不具合が判明。インデントは値展開前のパーシャル・テンプレート文字列自体に適用してからパース・レンダリングするよう修正した（BR-5.4修正）。

上記6・7はいずれもFunctional Design/Application Designで既に承認済みだった決定を、公式spec（NFR-2、本プロジェクトの最上位の正）との矛盾が実装・検証段階で判明したために上書きしたものであり、他の補正（Value/Map、MAX_NESTING_DEPTH等）よりも重大な設計変更である。承認済み上位ドキュメントを覆す点を明示的に記録する。

### Step 9: PBT Test Generation
- [x] `tests/proptest/`配下に、`business-logic-model.md`のTestable Propertiesテーブルに基づくプロパティテストを実装（`support.rs`に共通の`arb_value`ジェネレータとHTMLアンエスケープヘルパーを配置。内部AST（`Node`）は非公開のため、いずれも公開API（`Mustache::render_str`等）のみを用いて検証）:
  - `text_passthrough.rs`: テキストのみテンプレートの透過性（Invariant）
  - `section_nesting.rs`: セクション入れ子構造の保存（Induction。任意の深さのネストしたセクションが正しく開始・終了タグ対応し、内側の内容が過不足なく1回だけ出力されることを検証）
  - `escape_roundtrip.rs`: HTMLエスケープ/アンエスケープの往復（Round-trip）
  - `section_complement.rs`: セクション/逆セクションの相補性（Invariant、`arb_value`で生成した任意のValueに対して検証）
  - `array_repeat.rs`: 配列セクションの繰り返し回数（Invariant）
  - `partial_recursion_guard.rs`: 無限パーシャル再帰時の終端保証（Invariant、深度ガードのみに一本化。旧「パーシャル循環検出時の終端保証」から改称）
  - `directory_resolver_idempotence.rs`: DirectoryPartialResolverの解決結果安定性（Idempotence）
- [x] `nfr-design-patterns.md`パターン6に従い、軽量プロパティ（text_passthrough/escape_roundtrip/section_complement/array_repeat）はデフォルト256ケース、重いプロパティ（section_nesting/partial_recursion_guard/directory_resolver_idempotence）は64ケースに設定
- [x] `cargo test --test proptest`で7件全て成功を確認

### Step 10: Documentation Generation
- [x] `src/lib.rs`のクレートレベルdocコメントに使用例（`Mustache::new().render_str(...)`）を記載（Step6で先行実施済み、doctestとして`cargo test --doc`でも検証済み）
- [x] `cargo doc --no-deps`がエラーなく完了することを確認

### Step 11: Business Logic Summary
- [x] `aidlc-docs/construction/core-engine/code/summary.md`に生成物一覧・テスト構成・spec準拠状況をまとめる

**N/A（core-engineに該当なし）**: API Layer Generation, Repository Layer Generation, Frontend Components Generation, Database Migration Scripts, Deployment Artifacts Generation — core-engineはデータベース・API・UIを持たないライブラリのため対象外。配布形態（シングルバイナリ）はcliユニットのCode Generationおよび将来のOperations Phaseで扱う。
