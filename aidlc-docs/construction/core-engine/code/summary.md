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

## 未対応・対象外

- ラムダ、テンプレート継承等のオプションモジュール（FR-4/Q3=Bにより対象外）
- ストリーミング出力API（NFR Requirements Q1=Aにより対象外）

## 次のステップ

cliユニットのCONSTRUCTION（Functional Design → NFR Requirements → NFR Design → Code Generation）へ進む。
