# Tech Stack Decisions — core-engine

## 言語・ツールチェーン
- Rust, edition 2024
- 開発環境で確認済みのrustc/cargoバージョン: 1.97.0（`requirements.md` Technical Context）
- `Cargo.toml`に`rust-version`（MSRV）フィールドは特に固定しない。edition 2024を使用するため、MSRVは事実上edition 2024をサポートする最小バージョンに従う

## 依存クレート（core-engineユニット、`unit-of-work-dependency.md`と整合）

| クレート | 種別 | 用途 | 選定理由 |
|---|---|---|---|
| `serde` | 通常依存 | `Value`が受け入れ可能な型の`Serialize`トレイト境界 | Rustエコシステム標準のシリアライズ抽象化。JSON/YAML変換はcli側（`serde_json`/`serde_yaml`）に閉じるため、core-engineはトレイト境界のみに依存する |
| `proptest` | 開発依存（dev-dependency） | PBT-01で識別したTestable Propertiesの実装（NFR-3, PBT-09） | Q4=Bで正式採用。Rust向けPBTフレームワークとして`requirements.md`で推奨済み。マクロベースのAPIとshrinking機能を備える |

## 静的解析・Lint設定
- `lib.rs`クレートルートに`#![deny(missing_docs)]`を設定し、公開APIのrustdocコメント欠落をビルドエラーとする（Q3=B）
- `Parser`/`Renderer`等の非公開モジュールにはこのlintは適用されない（`pub`項目のみが対象）

## PBT実行方針（PBT-08, PBT-09関連）
- `proptest`のデフォルトのshrinking機能を無効化しない（PBT-08準拠）
- プロパティごとの試行回数（`ProptestConfig::cases`）:
  - 軽量なプロパティ（例: HTMLエスケープの往復、セクション/逆セクションの相補性）: デフォルト256ケース
  - 構造化されたテンプレート生成を伴う重いプロパティ（例: Parserの入れ子構造保存、パーシャル循環検出）: 64ケース程度に調整し、CI実行時間を抑制する（具体的な調整値はCode Generation時のテスト実装で確定する）
- 失敗時のシード値はテスト出力に含める（`proptest`標準機能、追加設定不要）

## 除外した選択肢
- ストリーミング出力用クレート（例: `io::Write`向けの追加抽象化）: Q1=AによりString返却のみで十分と判断したため導入しない
- 正規表現エンジン（`regex`等）: Mustacheのタグ検出は単純な文字列走査で実装可能であり、正規表現は不要と判断
