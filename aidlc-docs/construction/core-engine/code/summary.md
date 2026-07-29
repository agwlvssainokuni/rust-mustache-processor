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

## ブロック再インデント処理 フルサポート（v0.2.1、要記録）

`core-engine-block-reindentation-code-generation-plan.md`（全13ステップ）に基づき、v0.2.0でBR-10.7として先送りしていた「既知の制限」4ケースに対応した。公式`mustache/spec`リポジトリのIssue #130（フィクスチャ追加PR #131の設計議論）からRule1〜4のアルゴリズムを取得し、業務ルールとして確定（BR-10.7〜BR-10.13、詳細は`functional-design/business-rules.md`）。

### 生成物一覧（変更分）

| ファイル | 行数（v0.2.1時点） | 変更内容 |
|---|---|---|
| `src/ast.rs` | 92 | `Node::Block`に`raw`/`open_clears_start`/`open_clears_end`/`close_clears_start`/`close_clears_end`/`open_indent`を追加 |
| `src/parser.rs` | 978 | `Frame::Block`に`open_start`/`open_end`を追加、`clears_at_start`/`clears_at_end`/`leading_whitespace_before`ヘルパーを追加し、`Node::Block`構築時に新フィールドを算出 |
| `src/renderer.rs` | 1337 | `RenderState.block_overrides`の型を`Vec<HashMap<String, Node>>`に変更、`substitute_blocks`系（事前一括置換）を廃止し、`render_block`（`find_effective_override`・`dedent_block`・`expansion_indent_for`）による遅延解決方式に置き換え |
| `tests/spec/conformance.rs` | — | `inheritance_known_limitations`（`#[ignore]`）と`INHERITANCE_KNOWN_LIMITATIONS`定数を削除し、`inheritance()`を27ケース全件に統合 |
| `Cargo.toml` | — | versionを`0.2.0`→`0.2.1`に更新 |

### テスト（v0.2.1時点）

| 種別 | 件数 |
|---|---|
| ユニットテスト（`#[cfg(test)]`） | 93件（v0.2.0時点84件から9件追加: parser.rs 4件、renderer.rs 5件） |
| 公式spec conformanceテスト | 必須6モジュール136ケース + オプション3モジュール58ケース（`~inheritance`が23/27→27/27に。`#[ignore]`なし）= 計194ケース、全件合格 |

### Spec準拠状況（v0.2.1）

- 必須6モジュール: 136/136（100%）
- `~lambdas`: 10/10（100%）
- `~dynamic-names`: 21/21（100%）
- `~inheritance`: 27/27（100%、既知の制限を解消）

### Code Generation中に発見・修正した設計補正（v0.2.1）

1. **Rule4 Step2の二重適用バグ**（Step9、`cargo test --test spec`で発見）: スタンドアロンペア判定（`open_clears_start`かつ`close_clears_end`）による`open_indent`付与は、開始タグと同じ行にある差し替え前の既定内容が非空白を含む場合、既存のPass2（行単位スタンドアロン判定）ではその行がトリミングされず、開始タグ直前の空白が既にリテラル出力に残っている。これに気づかず`open_indent`をさらに付与したため、既存の合格ケース"Inherit indentation"で二重インデント（期待2スペースに対し実際4スペース）の回帰が発生。開始タグと同じ行にある`raw`内容が空白のみの場合に限りStep2を適用するよう修正して解消
2. **自前テストの転記ミス**（Step7）: 多段継承（Nested block reindentation）を検証する単体テストで、`grandparent`パーシャルの文字列に実在しない末尾`\n`を書いてしまい失敗。公式フィクスチャJSON（`~inheritance.json`）と直接突き合わせて修正
3. 上記2点を除き、Functional Designで設計した「定義箇所での除去→展開箇所での再付与→デフォルトデリミタでの再パース」の方式（生テキストへの事前適用によりネストが自然にカスケードする設計）は、4ケース全てで手動導出通りに動作した

### 未対応・対象外（v0.2.0時点）

- ブロックの再インデント処理（BR-10.7、既知の制限。フォローアップ課題）
- ストリーミング出力API（NFR Requirements Q1=Aにより対象外）

### 次のステップ

v0.2.0としてリリースする。ブロック再インデント処理は別途フォローアップ課題として着手する。
