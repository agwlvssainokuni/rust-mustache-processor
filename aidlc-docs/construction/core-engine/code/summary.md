# Code Generation Summary — core-engine

`core-engine-code-generation-plan.md`（全11ステップ）に基づき実装したcore-engineユニットの生成物一覧、テスト構成、spec準拠状況をまとめる。

## 生成物一覧

### ライブラリ本体（`src/`, クレート名`mustache_processor`）

| ファイル | 行数 | 内容 |
|---|---|---|
| `src/lib.rs` | 198 | クレートルート。`Template`/`Mustache`（公開エントリーポイント）、`#![deny(missing_docs)]` |
| `src/value.rs` | 804 | `Value`列挙型、`Map`（キー順序保持）、`is_truthy`/`get`/`iter`、`from_serialize`（カスタム`serde::Serializer`実装） |
| `src/ast.rs` | 44 | `SourcePosition`、`Node`（内部AST、非公開） |
| `src/parser.rs` | 641 | 3パス構成のパーサー（tokenize→行単位スタンドアロン判定→木構築） |
| `src/renderer.rs` | 756 | `RenderState`、変数展開・セクション評価・パーシャル解決・ネスト深度ガード |
| `src/partial.rs` | 86 | `PartialResolver`トレイト、`DirectoryPartialResolver` |
| `src/error.rs` | 144 | `ParseError`/`RenderError`/`Error`とその`*Kind`列挙型 |

`Cargo.toml`: `[lib] name = "mustache_processor"`を追加、依存に`serde`（通常）、`proptest`/`serde_json`（開発依存）を追加。

### テスト（`tests/`）

| 種別 | 場所 | 件数 |
|---|---|---|
| ユニットテスト（`#[cfg(test)]`、`src/`内） | value.rs/parser.rs/renderer.rs/partial.rs/lib.rs | 72件 |
| 公式spec conformanceテスト | `tests/spec/`（`main.rs` + `conformance.rs` + `fixtures/*.json`） | 6モジュール・136ケース |
| プロパティベーステスト（proptest） | `tests/proptest/`（8ファイル） | 7プロパティ |
| doctest | `src/lib.rs`クレートdoc | 1件 |

**合計: 86テスト実行単位（proptestは内部でケース数分の試行を実施）、全て成功**。

## Spec準拠状況

公式mustache/spec（<https://github.com/mustache/spec>）の必須6モジュール全fixtureに対し、`cargo test --test spec`で100%（136/136）成功することを確認済み:

- comments（12ケース）
- delimiters（14ケース）
- interpolation（42ケース、暗黙のイテレータ・ドット区切り名前を含む）
- inverted（22ケース）
- partials（12ケース、循環しない自己再帰・スタンドアロンインデントを含む）
- sections（34ケース、暗黙のイテレータ・ドット区切り名前を含む）

ラムダ（`~lambdas.json`）等の非必須（tilde接頭辞）モジュールはrequirements.md FR-4/Q3=Bの決定通り対象外。

## Code Generation中に発見・修正した主な設計補正

いずれも実装・spec conformanceテストの過程で発見し、`core-engine-code-generation-plan.md`の各Stepに詳細を記録済み:

1. **Value/Map**（Step1-2）: Application Design承認済みの`Value`メソッド（`from_serialize`/`is_truthy`/`get`/`iter`）とキー順序保持`Map`を、Functional Designが見落としていたため補正
2. **真偽判定**（Step2）: 空文字列・空Mapは公式spec上truthyであり、Application Designの要約記述（falsyと誤記）ではなくFunctional Designの精査済み記述を正として実装
3. **ネスト深度上限**（Step4）: NFR Design例示値1000は実スタックサイズ（Windows既定1MiB相当）で安全マージンなく溢れることが実測で判明し、100に修正
4. **パーシャル内容自体の構文エラー**（Step4）: `RenderErrorKind::PartialParseError`を追加
5. **公式spec conformanceによる7件の重大補正**（Step8）: 暗黙のイテレータ`{{.}}`、ドット区切り名前、複数タグ/`\r\n`を考慮したスタンドアロン判定の行単位化、スカラー真値セクションでのコンテキストプッシュ、パーシャル未解決のデフォルト空文字列化（strictモード時のみエラー）、パーシャル循環検出の削除（深度ガードのみに一本化）、パーシャルインデントの値展開前適用

## 承認後の追加補正（v0.1.1、要記録）

Code Generation承認後、ユーザーから「ライブラリ利用者の推移的依存を最小限にしたい」との要望を受け対応。詳細は`nfr-requirements/tech-stack-decisions.md`の該当節を参照。

- `Cargo.toml`の`clap`/`serde_json`/`serde_norway`を`optional = true`にし、`cli` feature（`default = ["cli"]`）にゲート。`[[bin]]`に`required-features = ["cli"]`を追加
- `cargo build --lib --no-default-features`のクリーンビルドで、ライブラリが`serde`系クレートのみに依存し`clap`等を一切コンパイルしないことを実測確認
- パッケージversionを`0.1.0`→`0.1.1`にパッチアップ
- README.md/README.en.mdの「ライブラリとしての使い方」節に`default-features = false`の指定方法と効果を追記

## 未対応・対象外（v0.1.1時点）

- ラムダ、テンプレート継承等のオプションモジュール（v0.2.0で対応。下記参照）
- ストリーミング出力API（NFR Requirements Q1=Aにより対象外）

## 次のステップ

cliユニットのCONSTRUCTION（Functional Design → NFR Requirements → NFR Design → Code Generation）へ進む。

## Mustacheオプションモジュール フルサポート（v0.2.0、要記録）

`core-engine-mustache-optional-modules-code-generation-plan.md`（全13ステップ）に基づき、`~lambdas`・`~inheritance`・`~dynamic-names`の3オプションモジュールを実装した。詳細な設計判断は`functional-design/business-rules.md`のBR-9〜BR-11を参照。

### 生成物一覧（変更・追加分）

| ファイル | 行数（v0.2.0時点） | 変更内容 |
|---|---|---|
| `src/value.rs` | 901 | `Value::Lambda(Rc<dyn Fn(&str) -> String>)`追加、`Debug`/`Clone`/`PartialEq`を手動実装に変更 |
| `src/ast.rs` | 78 | `PartialName`（Static/Dynamic）、`Node::Section`に`raw`/`open`/`close`追加、`Node::Parent`/`Node::Block`追加 |
| `src/parser.rs` | 821 | 親タグ（`<`）・ブロックタグ（`$`）・動的パーシャル（`>*`）の判定、Frame方式への書き換え、セクション生テキスト・デリミタの記録 |
| `src/renderer.rs` | 1138 | ラムダ呼び出し（インターポレーション/セクション文脈）、テンプレート継承（オーバーライド・スタックによる多段継承伝播）、動的パーシャル名解決 |
| `tests/spec/conformance.rs` | — | `~lambdas`（10ケース、Rubyコードを手動翻訳）・`~inheritance`・`~dynamic-names`のテスト関数を追加 |
| `tests/spec/fixtures/~lambdas.json`, `~inheritance.json`, `~dynamic-names.json` | — | mustache/specリポジトリから取得 |
| `tests/proptest/lambda_recursion_guard.rs`, `block_default_matches_partial.rs` | — | v0.2.0向けTestable Propertiesの実装 |
| `Cargo.toml` | — | versionを`0.1.1`→`0.2.0`に更新 |

### テスト（v0.2.0時点）

| 種別 | 件数 |
|---|---|
| ユニットテスト（`#[cfg(test)]`） | 84件（v0.1.1時点72件から12件追加） |
| 公式spec conformanceテスト | 必須6モジュール136ケース + オプション3モジュール58ケース（`~lambdas`10、`~dynamic-names`21、`~inheritance`27）= 計194ケース |
| プロパティベーステスト（proptest） | 9プロパティ（v0.1.1時点7件から2件追加） |
| doctest | 1件 |

### Spec準拠状況（v0.2.0）

- 必須6モジュール: 136/136（100%）
- `~lambdas`: 10/10（100%）
- `~dynamic-names`: 21/21（100%）
- `~inheritance`: 23/27（約85%、既知の制限あり。下記参照）

### Code Generation中に発見・修正した主な設計補正（v0.2.0）

1. **ラムダの再パースデリミタが文脈依存**（Step1、fixture精査）: インターポレーション文脈は常にデフォルトデリミタ、セクション文脈はそのタグ自身が有効だった時点のデリミタを使う（当初は一律「現在のデリミタ」としていたBR-9.3の誤りを修正）。`Node::Section`に`open`/`close`フィールドを追加
2. **ラムダはキャッシュしない**（Step1、fixture精査、BR-9.3b追加）: 同一ラムダへの複数回参照は都度呼び出す
3. **スタンドアロン判定・Parentのindent欠落**（Functional Designレビュー）: BR-7.1が継承タグ・ブロックタグ・動的パーシャルタグを対象に含めていなかった漏れ、`Node::Parent`に`indent`フィールドが欠落していた漏れを承認前に発見・修正
4. **多段継承のオーバーライド伝播**（Step8、`~inheritance.json`の"Recursion"フィクスチャで発見）: 当初は`{{<parent}}`解決のたびに一度だけツリー置換する設計だったが、最も外側の呼び出し元のオーバーライドが途中の階層を経ても優先されない不具合が判明。`RenderState`にオーバーライド・スタック（`block_overrides`）を持たせ、`{{<parent}}`解決のたびにフレームをpush/popし、実効オーバーライドをスタック全体から外側優先でマージする方式に修正（BR-10.5確定）
5. **既知の制限（BR-10.7）**: ブロックの「再インデント処理」（差し替え内容のインデントを定義箇所で除去し展開箇所で再付与する処理）は未実装。`Standalone block`・`Block reindentation`・`Intrinsic indentation`・`Nested block reindentation`の4ケースが未準拠。手計算による検証を重ねたが末尾改行の扱い等で不確実性が残ったため、ユーザーと相談の上、実装コストと得られる価値のバランスを鑑みフォローアップ課題として先送りした

### 未対応・対象外（v0.2.0時点）

- ブロックの再インデント処理（BR-10.7、既知の制限。フォローアップ課題）
- ストリーミング出力API（NFR Requirements Q1=Aにより対象外）

### 次のステップ

v0.2.0としてリリースする。ブロック再インデント処理は別途フォローアップ課題として着手する。
