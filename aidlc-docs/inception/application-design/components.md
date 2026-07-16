# Components — rust-mustache-processor

設計判断（application-design-plan.md: Q1=C, Q2=B, Q3=A, Q4=B, Q5=B）に基づくコンポーネント一覧。

## ユニット: core-engine（ライブラリ）

### Value（データモデル）
- **目的**: Mustacheのコンテキストとして扱う値を表現する、フォーマット非依存の内部データ型（FR-3, Application Design Q4=B）
- **責務**:
  - Null / Bool / Number / String / Array / Map（キー順序を保持するMap）のバリアントを持つ
  - `serde::Serialize`を実装した任意のRust型からの変換をサポートし、ライブラリ利用者がJSON/YAMLに依存せず値を構築できるようにする
  - Mustacheの真偽判定（truthy/falsy）ルールを提供する（false, null, 空文字列, 空配列, 空Mapはfalsy）
- **インターフェース**: 公開データ型。他の全コンポーネントから参照される

### Parser（パーサー）
- **目的**: Mustacheテンプレート文字列を構文解析し、AST（ノード列）を生成する（FR-4）
- **責務**:
  - 変数展開（エスケープあり/なし）、セクション、逆セクション、パーシャル、コメント、デリミタ変更のタグを認識する
  - デリミタ変更（`{{=<% %>=}}`）を解析中に適用し、以降のトークン化に反映する
  - 構文エラーを`ParseError`として報告する（Application Design Q5=B）
- **インターフェース**: `parse(template: &str) -> Result<Vec<Node>, ParseError>`（`Mustache::parse`から内部的に呼び出される）

### Template（コンパイル済みテンプレート）
- **目的**: パース済みAST（ノード列）を保持し、再利用可能な形で表現する（Application Design Q1=C）
- **責務**: ASTの所有のみ。レンダリングロジックは持たない（レンダリングは`Mustache`エンジンが`Renderer`を介して行う）
- **インターフェース**: 不透明な構造体。`Mustache::parse`の戻り値として生成される

### Renderer（レンダリングエンジン）
- **目的**: `Template`のASTを`Value`コンテキストに対して評価し、出力文字列を生成する（FR-4, FR-7）
- **責務**:
  - コンテキストスタックの管理（セクションに入る際のpush、抜ける際のpop）
  - セクション/逆セクションの真偽判定・配列繰り返し評価
  - 変数展開時のHTMLエスケープ処理（エスケープなしタグは素通し）
  - 未定義変数の挙動制御（デフォルト空文字継続 / strictモードでエラー、FR-7）
  - パーシャルタグに到達した時点で`PartialResolver`を呼び出し、取得したテンプレート文字列を`Parser`で解析して再帰的にレンダリングする（Application Design Q3=A: 遅延評価）
  - レンダリング時エラーを`RenderError`として報告する（Application Design Q5=B）
- **インターフェース**: 内部コンポーネント。`Mustache::render`から呼び出される

### PartialResolver（トレイト）
- **目的**: パーシャル名から対応するテンプレート文字列を解決する抽象インターフェース（Application Design Q2=B）
- **責務**: 名前解決の方法を実装側に委ねる（ディレクトリベース、メモリ上のマップ等）
- **インターフェース**: `trait PartialResolver { fn resolve(&self, name: &str) -> Option<String>; }`

### DirectoryPartialResolver（標準実装）
- **目的**: `PartialResolver`のファイルシステムディレクトリベース実装。IO専用ロジックのためcore-engineに同梱する（JSON/YAMLのようなデータフォーマット解析ではないためQ4の対象外）
- **責務**: 指定ディレクトリ配下から`{name}`に対応するファイル（拡張子は規約で定める）を読み込む
- **インターフェース**: `DirectoryPartialResolver::new(dir: PathBuf) -> Self`、`PartialResolver`トレイトを実装

### Mustache（エンジン/設定オブジェクト）
- **目的**: パーシャルリゾルバやstrictモード等の設定を保持し、パース・レンダリングの起点となる（Application Design Q1=C）
- **責務**:
  - 設定（`partial_resolver`, `strict`）の保持
  - `Parser`を介したテンプレートのパース
  - `Renderer`を介したレンダリングの実行
  - 一括処理用の簡易メソッド（パース＋レンダリングを1回で行う）も提供する
- **インターフェース**: 公開エントリーポイント（詳細は`component-methods.md`参照）

### Error型群（ParseError / RenderError）
- **目的**: パース時とレンダリング時のエラーを型で区別する（Application Design Q5=B）
- **責務**: 各エラー種別（構文エラー、未定義変数、パーシャル未解決等）の表現と、エラー発生箇所（行/列等）の情報保持
- **インターフェース**: 公開エラー型。`std::error::Error`を実装

## ユニット: cli（バイナリ）

### CliArgs（コマンドライン引数）
- **目的**: `mustache`コマンドの引数を解析し、実行に必要な設定へ変換する（FR-5）
- **責務**: テンプレート/データ/出力先の指定方法（ファイルパス、標準入出力切替）、`--partials-dir`、`--strict`等のオプション解析
- **インターフェース**: argv → `CliArgs`構造体

### IoController（入出力制御）
- **目的**: テンプレート・データの読み込み元、出力先を解決し、実際のバイト列を取得・書き出す（FR-5）
- **責務**:
  - データはデフォルトで標準入力、テンプレートはデフォルトでファイル引数。オプション指定でテンプレート側も標準入力に切替可能
  - テンプレート・データ双方が標準入力に指定された場合はエラーを返す
  - パーシャルディレクトリ未指定時、テンプレートファイルのディレクトリをデフォルトとして解決する（FR-6）

### DataLoader（データローダー）
- **目的**: JSON/YAMLのデータ文字列をパースし、core-engineの`Value`型に変換する（FR-3, Application Design Q4=B）
- **責務**:
  - 拡張子または明示オプションによる形式判定
  - `serde_json`/`serde_yaml`でのパースと、`Value`への変換

### CliRunner（オーケストレーション）
- **目的**: CLIのメインフローを制御する（FR-5〜FR-7）
- **責務**: `CliArgs`解析 → `IoController`でテンプレート/データ読み込み → `DataLoader`でデータ変換 → core-engineの`Mustache`エンジンでパース・レンダリング → `IoController`で出力 → エラー時は標準エラー出力へメッセージを出し、非ゼロ終了コードで終了