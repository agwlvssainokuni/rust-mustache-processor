# Requirements — Mustacheオプションモジュール フルサポート

## Intent Analysis

- **User Request**: "Mustacheのオプション機能を実装してフルサポートしたい。"
- **Request Type**: New Feature / Enhancement（既存`core-engine`ユニットの機能拡張。新規ユニットではない）
- **Scope Estimate**: Multiple Components（`value.rs`/`ast.rs`/`parser.rs`/`renderer.rs`/`error.rs`および関連テスト・ドキュメント一式に影響）
- **Complexity Estimate**: Complex（3つの独立したオプションモジュールそれぞれが固有の構文・評価規則を持ち、既存のAST/パーサー/レンダラーの拡張が必要）

既存の`requirements.md`（FR-4）は、公式[mustache/spec](https://github.com/mustache/spec)のオプションモジュール（`~`接頭辞）である`~lambdas`・`~inheritance`・`~dynamic-names`をv1スコープから明示的に除外していた。本要件は、その除外事項を撤回し、フルサポートを実現するものである。

## 機能要件（Functional Requirements）

### FR-1: 実装対象モジュール
以下3つのオプションモジュールすべてを実装し、公式spec準拠の対象に追加する。
- `~lambdas`（ラムダ）
- `~inheritance`（テンプレート継承）
- `~dynamic-names`（動的パーシャル名）

### FR-2: ラムダ — 提供形態
ラムダは`Value`列挙型に新しいバリアント（例: `Value::Lambda`）として追加し、Rustのクロージャ/関数ポインタをラップして保持する。JSON/YAMLのデータ入力からは構築できないため、**ライブラリAPI経由でのみ**利用可能とする。CLIからのラムダ利用は対象外とする（FR-2の制約により原理的に不可能）。

### FR-3: ラムダ — spec準拠レベル
公式spec fixture（`~lambdas.json`）に100%準拠する。具体的には以下を含む:
- インターポレーション文脈（`{{lambda}}`）: 引数なしで呼び出し、返り値をMustacheテンプレートとして再パース・再レンダリングした上で（HTMLエスケープの要否は通常の変数展開と同じ規則で）出力に埋め込む
- セクション文脈（`{{#lambda}}...{{/lambda}}`）: セクション内の未展開の生テンプレート文字列を引数として呼び出し、返り値を現在のコンテキストで再帰的にレンダリングして出力に埋め込む
- 逆セクション文脈（`{{^lambda}}...{{/lambda}}`）: ラムダは常にtruthyとして扱われ、逆セクションの中身は描画されない

### FR-4: テンプレート継承 — 構文
以下の構文をサポートする。
- `{{<parent}}...{{/parent}}`: `parent`という名前のテンプレートを親として継承する
- `{{$block}}...{{/block}}`: 親テンプレート側でデフォルト内容を定義するブロック
- 子テンプレート側の`{{$block}}...{{/block}}`（`{{<parent}}`の内側）は、親テンプレートの同名ブロックの内容を上書きする

### FR-5: テンプレート継承 — 親テンプレートの解決
親テンプレートの名前解決は、既存の`PartialResolver`トレイト（ディレクトリベース解決）をそのまま流用する。継承専用の解決の仕組みは新設しない。

### FR-6: 動的パーシャル名 — 構文
`{{>* partialNameVar}}`構文をサポートする。`partialNameVar`をコンテキストから解決した文字列値をパーシャル名として、通常のパーシャル解決（`PartialResolver`）に渡す。

### FR-7: 動的パーシャル名 — エラー処理
`partialNameVar`の値が文字列でない場合や未定義の場合は、既存の「未解決パーシャル」と同じ扱いにする（非strictモードでは空文字列として継続、strictモードではエラーとする）。

### FR-8: 既存要件の改訂
`requirements.md`のFR-4における「対象外: ラムダ...」の記述は、本要件により撤回・上書きされる。`requirements.md`側には本ドキュメントへの参照を追記する。

## 非機能要件（Non-Functional Requirements）

### NFR-1: spec conformanceテストの拡張
`~lambdas.json`・`~inheritance.json`・`~dynamic-names.json`の3オプションfixtureを`tests/spec/fixtures/`に取り込み、既存の必須6モジュールと同じ枠組み（`tests/spec/conformance.rs`）で100%準拠を検証する。

### NFR-2: バージョニング
SemVerに従い、マイナーバージョンアップとする（`0.1.1` → `0.2.0`）。新機能追加であり後方互換性は維持される（既存のMustache構文・ライブラリAPIに破壊的変更はない）。

### NFR-3: ドキュメント整合性
README.md/README.en.mdの「対応していない機能」の記述を更新し、実装後は3モジュールが公式spec準拠していることを明記する。ライブラリ利用者向けにラムダAPIの使用例を追記する。

## スコープ外（Out of Scope）

- CLIからのラムダ利用（FR-2の制約により原理的に不可能。JSON/YAMLに関数値を表現する手段がないため）
- ラムダ以外の新しい`Value`計算バリアント（本要件はMustache公式spec準拠のラムダのみを対象とする）

## Summary of Key Decisions

| 項目 | 決定内容 |
|---|---|
| 実装対象 | `~lambdas`・`~inheritance`・`~dynamic-names`の3モジュールすべて |
| ラムダの提供形態 | ライブラリAPI限定（CLIでは利用不可） |
| ラムダのspec準拠 | `~lambdas.json`に100%準拠 |
| 継承の親解決 | 既存`PartialResolver`を流用 |
| 動的パーシャル名のエラー処理 | 既存の未解決パーシャルと同じ扱い |
| テスト | 3モジュールの公式fixtureを取り込み100%準拠を検証 |
| バージョン | マイナーバージョンアップ（0.1.1 → 0.2.0） |

各決定の詳細な理由は`mustache-optional-modules-requirement-verification-questions.md`を参照。
