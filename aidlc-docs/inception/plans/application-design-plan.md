# Application Design Plan — rust-mustache-processor

## Design Scope

`requirements.md`（FR-1〜FR-8, NFR-1〜NFR-6）を踏まえ、以下2ユニットを対象にコンポーネント・メソッド・サービス層・依存関係を設計する。

- **core-engine**: Mustache処理エンジン（パース・データモデル・レンダリング・パーシャル解決）を提供するライブラリ
- **cli**: `mustache`コマンドとして提供するCLIバイナリ（core-engineの薄いラッパー）

## Plan Checklist

- [ ] Q&Aで未決定の設計判断を確定する（下記「設計判断のための質問」）
- [ ] `components.md` — コンポーネント定義と責務を生成
- [ ] `component-methods.md` — 各コンポーネントのメソッドシグネチャ（概要レベル、詳細な業務ルールはFunctional Designで定義）を生成
- [ ] `services.md` — サービス定義とオーケストレーションパターンを生成
- [ ] `component-dependency.md` — 依存関係マトリクスと通信パターンを生成
- [ ] `application-design.md` — 上記を統合した単一ドキュメントを生成
- [ ] 設計の完全性・一貫性を検証

## 設計判断のための質問

以下の質問に回答してください。各質問の`[Answer]:`タグの後に選択肢の記号を記入してください。該当がなければ最後の「Other」を選んで自由記述してください。

### Question 1: ライブラリの公開API形状
core-engineライブラリの主要な呼び出しインターフェースはどの形にしますか？

A) 単一関数呼び出し型 — `render(template: &str, data: Value, options: RenderOptions) -> Result<String, Error>`。都度テンプレート文字列を渡し、内部で毎回パースする、最もシンプルな形

B) 2段階型 — `Template::parse(&str) -> Result<Template>`でASTを構築して再利用可能にし、`.render(&data) -> Result<String>`を複数回呼べる形（同じテンプレートを繰り返しレンダリングする場合にパースコストを削減できる）

C) エンジン設定オブジェクト型 — パーシャルディレクトリやstrictモード等の設定を保持する`Mustache`エンジン/コンテキストオブジェクトを作り、そこからテンプレートのパース・レンダリングを行う（Bをベースに、設定の使い回しも可能にする最も柔軟な形）

X) Other (please describe after [Answer]: tag below)

[Answer]: 

### Question 2: パーシャル解決の抽象化レベル
パーシャル解決の実装をどの程度抽象化しますか？

A) ディレクトリベースの実装のみを直接組み込む（シンプル、v1のCLIユースケースに集中）

B) トレイト（`PartialResolver`のような抽象インターフェース）を定義し、ディレクトリベース実装はその1実装として提供する（ライブラリ利用者が独自のリゾルバ、例: メモリ上のテンプレート集合、を差し込める）

X) Other (please describe after [Answer]: tag below)

[Answer]: 

### Question 3: 存在しないパーシャル参照時の挙動タイミング
テンプレートが存在しないパーシャルを参照している場合、いつエラーにしますか？

A) レンダリング実行時に、実際にそのパーシャルタグへ到達した時点でエラーにする（未到達のセクション内であれば評価されずエラーにならない。Mustacheの遅延評価と一貫性がある）

B) レンダリング開始前に、テンプレート内の全パーシャル参照を事前検証し、一つでも解決できなければ即座にエラーにする（早期に問題を検知できる）

X) Other (please describe after [Answer]: tag below)

[Answer]: 

### Question 4: JSON/YAMLパース機能（データローダー）の配置
FR-3のJSON/YAML対応は、どのコンポーネントに実装しますか？

A) core-engineライブラリに含める（ライブラリ利用者もJSON/YAML文字列を直接渡せる。ただし`serde_json`/`serde_yaml`がライブラリの依存に含まれる）

B) cli側にのみ実装する（core-engineは既にパース済みの内部`Value`型のみを受け取るフォーマット非依存設計にする。ライブラリ利用者は`serde::Serialize`実装や独自変換で`Value`を用意する）

X) Other (please describe after [Answer]: tag below)

[Answer]: 

### Question 5: エラー型の粒度
ライブラリのエラー型はどの程度の粒度で分類しますか？

A) 単一のエラー型（enum）にパース・レンダリング・未定義変数・パーシャル未解決など全種別をまとめる（呼び出し側は`match`で分岐可能）

B) パースエラーとレンダリングエラーを別の型として分離する（`Template::parse`と`.render`それぞれ専用のエラー型を持ち、型シグネチャで区別を強制する）

X) Other (please describe after [Answer]: tag below)

[Answer]: 
