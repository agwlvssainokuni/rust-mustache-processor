# Services — rust-mustache-processor

このプロジェクトは分散サービスを持たない単一プロセスのライブラリ＋CLIであるため、「サービス層」は外部通信を伴わないオーケストレーション（ユースケース単位の処理フロー統括）を指す。

## RenderingService（core-engine内、`Mustache`エンジンが担う）

- **責務**: テンプレートのパースとレンダリングという2つの操作を、設定（パーシャルリゾルバ、strictモード）と紐づけて提供する
- **オーケストレーション対象**: `Parser` → `Template`（AST保持） → `Renderer`（`PartialResolver`を介した再帰的パーシャル解決を含む）
- **公開境界**: `Mustache`, `Template`, `Value`, `PartialResolver`（トレイト）, `DirectoryPartialResolver`, `ParseError`, `RenderError`, `Error`のみを公開APIとする。`Parser`/`Renderer`の内部実装はプライベートモジュールとし、cliを含む外部からは`Mustache`経由でのみ利用させる

## CliRenderService（cli内、`CliRunner`が担う）

- **責務**: CLI実行の一連の流れ（引数解析 → 入力読み込み → データ変換 → レンダリング → 出力）を統括する
- **オーケストレーション対象**:
  1. `CliArgs::parse_args` — 引数解析
  2. `IoController::read_template` / `read_data` — 入力読み込み（FR-5のstdin/ファイル切替ルールを適用）
  3. `DataLoader::detect_format` / `load` — JSON/YAMLをcore-engineの`Value`に変換
  4. `IoController::resolve_partials_dir` — パーシャルディレクトリのデフォルト解決（FR-6）
  5. core-engineの`Mustache`（`DirectoryPartialResolver`と`strict`設定を適用）で`parse` → `render`
  6. `IoController::write_output` — 結果の書き出し
  7. エラー発生時は標準エラー出力へメッセージを出力し、非ゼロ終了コードでプロセスを終了する
- **依存境界**: CliRenderServiceはcore-engineの公開API（`Mustache`, `Value`, `PartialResolver`実装等）のみに依存し、core-engineの内部モジュール（`Parser`, `Renderer`）には直接依存しない

## オーケストレーションパターンのまとめ

```
[cli] CliRunner
  └─ CliArgs::parse_args
  └─ IoController::read_template / read_data
  └─ DataLoader::detect_format / load          → core-engine::Value
  └─ IoController::resolve_partials_dir
  └─ core-engine::Mustache::new()
        .with_partial_resolver(DirectoryPartialResolver::new(dir))
        .with_strict(args.strict)
     └─ .parse(template_str)   → Template        [RenderingService]
     └─ .render(&template, &data) → String        [RenderingService]
           └─ Renderer（遅延パーシャル解決: 到達時にPartialResolver::resolveを呼び、
              取得した文字列をParserで解析し再帰的にレンダリング）
  └─ IoController::write_output
```