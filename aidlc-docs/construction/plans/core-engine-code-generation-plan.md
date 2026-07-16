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
- [ ] `src/renderer.rs`: `RenderState`（context_stack, depth, partial_chain, strict）、`MAX_NESTING_DEPTH`定数（1000）
- [ ] 変数展開（エスケープ有無、BR-1.1〜BR-1.9）、セクション/逆セクション評価（BR-2.1〜BR-3.1）、コンテキストスタック探索（BR-4.1〜BR-4.2）を実装
- [ ] パーシャル解決（BR-5.1〜BR-5.5: 遅延評価、常にエラー化、循環検出、インデント適用）とネスト深度制限（Recursion Guardパターン）を実装
- [ ] `src/error.rs`に`RenderError`, `RenderErrorKind`（UndefinedVariable, PartialNotFound, PartialCycleDetected, MaxNestingDepthExceeded）を追加

### Step 5: Business Logic Generation — PartialResolver / DirectoryPartialResolver
- [ ] `src/partial.rs`: `PartialResolver`トレイト（`resolve(&self, name: &str) -> Option<String>`、`component-methods.md`準拠）
- [ ] `DirectoryPartialResolver`（ディレクトリベース実装、`{name}.mustache`ファイルを読み込み）

### Step 6: Business Logic Generation — Mustache エンジン公開API
- [ ] `src/lib.rs`: `Template`（`root: Vec<Node>`, `source_len: usize`）、`Mustache`（`new`, `with_partial_resolver`, `with_strict`, `parse`, `render`, `render_str`）、`Error`型（Parse/Render統合）を実装
- [ ] `Mustache::render`で`String::with_capacity(template.source_len)`による事前確保（Capacity Pre-allocationパターン）を適用
- [ ] 全公開項目にrustdocコメントを付与（`#![deny(missing_docs)]`がビルドを通ることを確認）

### Step 7: Business Logic Unit Testing
- [ ] `value.rs`内に`#[cfg(test)]`ユニットテスト（is_truthy各パターン、get/iter、from_serialize）
- [ ] `parser.rs`内に`#[cfg(test)]`ユニットテスト（各タグ種別、デリミタ変更、スタンドアロントリミング、構文エラー）
- [ ] `renderer.rs`内に`#[cfg(test)]`ユニットテスト（エスケープ、セクション各パターン、strictモード、パーシャル未解決/循環/深度超過）

### Step 8: Spec Conformance Test Generation
- [ ] 公式mustache/specリポジトリ（`https://github.com/mustache/spec`）より必須モジュール（comments, delimiters, interpolation, inverted, partials, sections）のJSON定義を取得し`tests/spec/fixtures/`に配置
- [ ] `tests/spec/conformance.rs`: フィクスチャを読み込み、各テストケースをMustacheエンジンで実行し期待出力と比較する統合テストハーネスを実装（NFR-2）

### Step 9: PBT Test Generation
- [ ] `tests/proptest/`配下に、`business-logic-model.md`のTestable Propertiesテーブルに基づくプロパティテストを実装:
  - テキストのみテンプレートの透過性（Invariant）
  - セクション入れ子構造の保存（Induction）
  - HTMLエスケープ/アンエスケープの往復（Round-trip）
  - セクション/逆セクションの相補性（Invariant）
  - 配列セクションの繰り返し回数（Invariant）
  - パーシャル循環検出時の終端保証（Invariant）
  - DirectoryPartialResolverの解決結果安定性（Idempotence）
- [ ] `nfr-design-patterns.md`パターン6に従い、軽量プロパティはデフォルト256ケース、重いプロパティ（Parser構造保存、循環検出）は64ケースに設定

### Step 10: Documentation Generation
- [ ] `src/lib.rs`のクレートレベルdocコメントに使用例（`Mustache::new().render_str(...)`）を記載
- [ ] `cargo doc --no-deps`がエラーなく完了することを確認

### Step 11: Business Logic Summary
- [ ] `aidlc-docs/construction/core-engine/code/summary.md`に生成物一覧・テスト構成・spec準拠状況をまとめる

**N/A（core-engineに該当なし）**: API Layer Generation, Repository Layer Generation, Frontend Components Generation, Database Migration Scripts, Deployment Artifacts Generation — core-engineはデータベース・API・UIを持たないライブラリのため対象外。配布形態（シングルバイナリ）はcliユニットのCode Generationおよび将来のOperations Phaseで扱う。
