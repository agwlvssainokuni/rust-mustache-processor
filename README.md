# rust-mustache-processor

[English](README.en.md)

Rustでゼロから実装したMustacheテンプレートエンジンです。ライブラリ（`mustache_processor`）とCLIツール（`mustache`）の両方として利用できます。

- 公式[Mustache spec](https://github.com/mustache/spec)の必須機能一式に準拠（変数展開、セクション、逆セクション、パーシャル、コメント、デリミタ変更）
- 公式spec conformanceテスト（comments/delimiters/interpolation/inverted/partials/sections、計136ケース）で100%準拠を確認済み
- 入力データはJSON・YAMLの両方に対応
- 複数テンプレートファイルの連結（`cat`ライクな指定順連結）に対応
- 未定義変数をエラーにするstrictモード

## インストール

### ビルド済みバイナリを使う

[GitHub Releases](../../releases)から、お使いのOS・アーキテクチャ向けのアーカイブ（`mustache-<version>-<target-triple>.tar.gz`または`.zip`）をダウンロードし、展開して`mustache`（Windowsは`mustache.exe`）をPATHの通ったディレクトリに配置してください。

対応プラットフォーム: Linux (x86_64) / macOS (Apple Silicon) / Windows (x86_64)

### ソースからビルドする

```bash
git clone <this-repository>
cd rust-mustache-processor
cargo install --path .
```

`mustache`コマンドがインストールされます。

## CLIの使い方

```bash
mustache [OPTIONS] [TEMPLATES]...
```

### 基本的な使い方

```bash
# データは標準入力から読み込む（デフォルト）
mustache template.tmpl < data.json

# データファイルを明示指定
mustache template.tmpl --data data.json

# 出力先ファイルを指定（未指定時は標準出力）
mustache template.tmpl --data data.json --output result.txt
```

### 複数テンプレート

複数のテンプレートファイルを指定すると、指定順に個別にレンダリングした結果を連結します（`cat`と同様の挙動）。

```bash
mustache header.tmpl body.tmpl footer.tmpl --data data.json
```

### テンプレートを標準入力から読み込む

```bash
cat template.tmpl | mustache --template-stdin --data data.json
```

（`--template-stdin`と`--data`未指定＝標準入力の組み合わせは、両方が標準入力を要求するためエラーになります）

### パーシャル

パーシャル（`{{> partial}}`）は、`--partials-dir`未指定時はテンプレートファイル自身のディレクトリから解決されます（複数テンプレート指定時はファイルごとに解決）。

```bash
mustache template.tmpl --data data.json --partials-dir ./partials
```

### strictモード

未定義変数の参照をエラーにします（デフォルトは空文字列として継続）。

```bash
mustache template.tmpl --data data.json --strict
```

### データ形式

デフォルトはデータファイルの拡張子（`.json`/`.yaml`/`.yml`）から自動判定します。`--format`で明示指定も可能です。

```bash
mustache template.tmpl --data data.yaml
mustache template.tmpl --data data.txt --format yaml
```

### 全オプション一覧

```
Usage: mustache [OPTIONS] [TEMPLATES]...

Arguments:
  [TEMPLATES]...  テンプレートファイル（複数指定可、指定順に処理・連結される）

Options:
      --template-stdin               テンプレートを標準入力から読み込む（位置引数のテンプレートとは併用不可）
      --data <DATA>                  データファイル（未指定時は標準入力から読み込む）
  -o, --output <OUTPUT>              出力先ファイル（未指定時は標準出力へ書き出す）
      --partials-dir <PARTIALS_DIR>  パーシャル探索ディレクトリ（未指定時はテンプレートファイルごとのディレクトリ）
      --strict                       未定義変数の参照をエラーとするstrictモード
      --format <FORMAT>              データ形式を明示指定する（json または yaml）
  -h, --help                         Print help
  -V, --version                      Print version
```

## ライブラリとしての使い方

`Cargo.toml`:

```toml
[dependencies]
mustache_processor = { path = "../rust-mustache-processor", default-features = false }
```

`default-features = false`を指定すると、CLI専用の依存（`clap`/`serde_json`/`serde_norway`）を含まない最小限の依存関係（`serde`のみ）でライブラリを利用できます。指定しない場合はデフォルトの`cli` featureが有効になり、これらも推移的に含まれます。

```rust
use mustache_processor::Mustache;
use mustache_processor::value::{Map, Value};

let mustache = Mustache::new();
let mut data = Map::new();
data.insert("name", Value::String("World".to_string()));

let output = mustache
    .render_str("Hello, {{name}}!", &Value::Map(data))
    .unwrap();
assert_eq!(output, "Hello, World!");
```

任意の`serde::Serialize`実装型から`Value`へ変換することもできます（`Value::from_serialize`）。パーシャルやstrictモードはビルダースタイルで設定します。

```rust
use mustache_processor::Mustache;
use mustache_processor::partial::DirectoryPartialResolver;

let mustache = Mustache::new()
    .with_strict(true)
    .with_partial_resolver(Box::new(DirectoryPartialResolver::new("./partials")));
```

## 開発

```bash
cargo build          # ビルド
cargo test           # 全テスト実行（ユニットテスト・プロパティベーステスト・spec conformanceテスト）
cargo doc --no-deps  # ライブラリAPIドキュメント生成
```

対応していない機能（ラムダ等のオプションMustache拡張モジュール）や設計上の詳細な決定事項は`aidlc-docs/`配下のドキュメントを参照してください。

## ライセンス

Apache License 2.0（`LICENSE`参照）。Copyright agwlvssainokuni.
