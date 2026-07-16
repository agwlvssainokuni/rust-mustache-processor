# Tech Stack Decisions — cli

## 言語・ツールチェーン

core-engineと共通（`aidlc-docs/construction/core-engine/nfr-requirements/tech-stack-decisions.md`参照）: Rust edition 2024、rustc/cargo 1.97.0。

## 依存クレート（cliユニット、`unit-of-work-dependency.md`と整合）

| クレート | 種別 | 用途 | 選定理由 |
|---|---|---|---|
| `clap` | 通常依存 | `CliArgs`の引数解析 | derive API（`#[derive(Parser)]`）を採用（Q2=A）。Rustエコシステムで最も広く使われるCLI引数解析クレートで、ドキュメント・実績が豊富 |
| `serde_json` | 通常依存 | JSONデータのパース（`DataLoader`） | Rustエコシステム標準のJSON実装。`serde_json::Value`から`Value::from_serialize`でcore-engineの`Value`へ変換する |
| `serde_yaml` | 通常依存 | YAMLデータのパース（`DataLoader`） | `serde`エコシステムと親和性が高いYAML実装。`serde_json`と同様の変換パターンで扱える |
| `mustache_processor`（core-engine） | パス依存 | パース・レンダリングエンジン本体 | 同一パッケージ内のライブラリターゲット |

**`serde_json`の依存種別に関する注記**: core-engineのCode Generation（Step8）では、公式spec JSONフィクスチャ読み込みのため`serde_json`を`[dev-dependencies]`に追加済みだった（`core-engine/nfr-requirements/tech-stack-decisions.md`参照）。cliユニットでは`DataLoader`の実行時ロジックとして利用するため、cliのCode Generation時に`[dependencies]`（通常依存）へ昇格させる。Cargoの仕様上、`[dependencies]`に存在すればdev-dependency相当の用途にもそのまま使えるため、重複定義にはならない。

## 静的解析・Lint設定

- `main.rs`/`cli/`配下のモジュールには`#![deny(missing_docs)]`を適用しない（Q1=A、cliは公開APIを持たないバイナリのため）
- core-engine側の`#![deny(missing_docs)]`（`src/lib.rs`）には影響しない

## リリースビルド設定

`[profile.release]`の追加設定（LTO、strip、opt-level等）は行わない。Cargoのデフォルト設定を使用する（Q3=A）。

## PBT実行方針

- `proptest`を再利用する（core-engineで導入済み、Q4=A）
- cliのプロパティ（DataLoaderのJSON/YAML往復変換等）はファイルI/Oを伴わない軽量な変換のため、デフォルト256ケースとする

## 除外した選択肢

- `serde_yaml`以外のYAML実装（`yaml-rust`等）: `serde`エコシステムとの統合が`Value::from_serialize`の設計と直接整合するため`serde_yaml`を採用
- `clap`のbuilder API: derive APIを採用（Q2=A、理由は`nfr-requirements.md`参照）
- リリースビルド最適化設定: Q3=Aにより見送り
