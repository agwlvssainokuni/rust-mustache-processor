# Component Dependency — rust-mustache-processor

## 依存関係マトリクス

| コンポーネント | ユニット | 依存先 | 依存の種類 |
|---|---|---|---|
| Value | core-engine | （なし） | 純粋データ構造。`serde`（`Serialize`トレイト境界のみ）に依存 |
| Parser | core-engine | Value（Node内のリテラル値表現に一部利用する場合を除き基本非依存） | 構文解析ロジック |
| Template | core-engine | Parser（生成物としてのAST型を共有） | データ保持のみ |
| Renderer | core-engine | Value, Parser（パーシャル文字列を再帰的に解析するため）, PartialResolver（トレイト） | レンダリングロジック |
| PartialResolver（トレイト） | core-engine | （なし） | インターフェース定義 |
| DirectoryPartialResolver | core-engine | PartialResolver（トレイト実装）, `std::fs` | ファイルシステムIO |
| Mustache（エンジン） | core-engine | Parser, Renderer, PartialResolver, Template, Value, ParseError, RenderError | 公開ファサード。Parser/Rendererを内部的にオーケストレーション |
| ParseError / RenderError | core-engine | （なし） | エラー型定義 |
| CliArgs | cli | （なし） | argv解析（`clap`等の外部クレートを利用） |
| IoController | cli | CliArgs, `std::fs` / `std::io` | ファイル・標準入出力アクセス |
| DataLoader | cli | core-engine::Value, `serde_json`, `serde_yaml` | データフォーマット変換 |
| CliRunner | cli | CliArgs, IoController, DataLoader, core-engine::Mustache, core-engine::DirectoryPartialResolver | オーケストレーション |

## 通信パターン

- **cli → core-engine**: 関数呼び出しのみ（プロセス内、同期呼び出し）。cliはcore-engineの公開API（`Mustache`, `Template`, `Value`, `PartialResolver`, `DirectoryPartialResolver`, エラー型）のみを利用し、`Parser`/`Renderer`といった内部モジュールには直接依存しない
- **Renderer → PartialResolver**: トレイトオブジェクト経由の動的ディスパッチ。パーシャルタグに到達した時点で`resolve`を呼び出す（遅延評価、Application Design Q3=A）
- **Renderer → Parser**: パーシャル解決で得たテンプレート文字列を、レンダリング中に再帰的にパースするために呼び出す（パーシャル内にさらにパーシャルが含まれるケースに対応）
- **DataLoader → Value**: 変換のみの一方向依存（`serde_json::Value`/`serde_yaml::Value`からcore-engine::Valueへの変換関数）

## データフロー図（テキスト表現）

```
[argv]
  │
  ▼
CliArgs::parse_args
  │
  ├─→ IoController::read_template ──→ [template string]
  ├─→ IoController::read_data ──────→ [raw data string]
  │                                        │
  │                                        ▼
  │                              DataLoader::detect_format / load
  │                                        │
  │                                        ▼
  │                                  core-engine::Value
  │
  ├─→ IoController::resolve_partials_dir ──→ [partials dir path]
  │                                        │
  │                                        ▼
  │                          DirectoryPartialResolver::new
  │
  ▼
core-engine::Mustache::new()
  .with_partial_resolver(...)
  .with_strict(...)
  │
  ├─→ .parse([template string]) ──→ Template（AST）
  │        │
  │        ▼（内部）Parser::parse
  │
  └─→ .render(&Template, &Value) ──→ [output string]
           │
           ▼（内部）Renderer::render
             ├─ コンテキストスタック操作（セクション）
             ├─ 変数展開（エスケープ判定）
             └─ パーシャル遭遇時: PartialResolver::resolve → Parser::parse → 再帰render
  │
  ▼
IoController::write_output ──→ [stdout / output file]
```

## 依存関係の原則

- **単方向依存**: cli → core-engine のみ。core-engineはcliの存在を一切知らない（逆方向依存なし）
- **内部モジュールの非公開**: core-engineの`Parser`/`Renderer`はクレート内部（`pub(crate)`相当）とし、外部からは`Mustache`エンジンのAPIのみを経由させる
- **フォーマット依存の局所化**: `serde_json`/`serde_yaml`への依存はcli（DataLoader）に閉じ、core-engineには波及させない（Application Design Q4=B）