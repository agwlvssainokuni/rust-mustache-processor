# Mustacheオプションモジュール フルサポート — 要件確認質問

現行の`requirements.md`（FR-4）では、公式[mustache/spec](https://github.com/mustache/spec)のオプションモジュール（`~`接頭辞）である以下3つを対象外としています。

- `~lambdas`（ラムダ）
- `~inheritance`（テンプレート継承、`{{<parent}}`/`{{$block}}`）
- `~dynamic-names`（動的パーシャル名、`{{>* name}}`）

「フルサポート」の実現に向けて、以下の質問にお答えください。

## Question 1
実装対象とするオプションモジュールの範囲は？

A) 3モジュールすべて（ラムダ、テンプレート継承、動的パーシャル名）を実装し完全準拠とする

B) ラムダのみを実装する

C) テンプレート継承と動的パーシャル名のみを実装する（ラムダは対象外のまま）

D) Other (please describe after [Answer]: tag below)

[Answer]: 

## Question 2
ラムダのサポート範囲について。CLIの入力データ形式（JSON/YAML）では関数値を表現できないため、ラムダは事実上ライブラリAPI経由（Rustのクロージャを`Value`の新しいバリアントとして渡す）でのみ利用可能になります。CLIからは（データにラムダを含められないため）引き続き利用できません。この制約でよいですか？

A) はい、ライブラリAPI限定で問題ない

B) CLIからも何らかの形でラムダ相当の機能を使いたい（詳細は別途相談）

C) Other (please describe after [Answer]: tag below)

[Answer]: 

## Question 3
ラムダの公式spec準拠レベルについて。公式spec（`~lambdas.json`）は、インターポレーション文脈（`{{lambda}}`）とセクション文脈（`{{#lambda}}...{{/lambda}}`）で異なる呼び出し規約（引数の有無、返り値の再パース要否、HTMLエスケープの有無等）を定義しています。どこまで準拠しますか？

A) 公式spec fixture（`~lambdas.json`）に100%準拠する（既存の必須6モジュールと同じ品質基準）

B) 基本的なインターポレーション文脈のみ対応する（セクション文脈での返り値の再帰的レンダリングは対象外とする）

C) Other (please describe after [Answer]: tag below)

[Answer]: 

## Question 4
テンプレート継承（`{{<parent}}...{{/parent}}`で親を指定し、`{{$block}}...{{/block}}`でブロックを差し替える）における親テンプレートの解決方法は？

A) 既存の`PartialResolver`トレイト（ディレクトリベース解決）をそのまま流用する（親の指定名はパーシャル名と同じ解決規則に従う）

B) 継承専用の別の解決の仕組みを新設する

C) Other (please describe after [Answer]: tag below)

[Answer]: 

## Question 5
動的パーシャル名（`{{>* partialNameVar}}`）について、変数の値が文字列でない場合（Map、配列、数値等）や未定義の場合の扱いは？

A) 既存の「未解決パーシャル」と同じ扱いにする（strictモードはエラー、非strictモードは空文字列として継続）

B) 値が文字列でない場合は常にエラーとする（strict/非strictを問わず）

C) Other (please describe after [Answer]: tag below)

[Answer]: 

## Question 6
テストについて、公式spec conformanceの追加方針は？

A) 3モジュールのオプションfixture（`~lambdas.json`、`~inheritance.json`、`~dynamic-names.json`）を`tests/spec/fixtures/`に取り込み、必須6モジュールと同様100%準拠を確認する

B) 独自テストケースのみで対応し、公式fixtureは取り込まない

C) Other (please describe after [Answer]: tag below)

[Answer]: 

## Question 7
バージョニングについて、この機能拡張はどのように扱いますか？

A) マイナーバージョンアップとする（0.1.1 → 0.2.0、SemVerに従い新機能追加としてminorを上げる）

B) パッチバージョンアップとする（0.1.1 → 0.1.2）

C) Other (please describe after [Answer]: tag below)

[Answer]: 
