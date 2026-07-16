# Tech Stack Decisions — cli

## 言語・ツールチェーン

core-engineと共通（`aidlc-docs/construction/core-engine/nfr-requirements/tech-stack-decisions.md`参照）: Rust edition 2024、rustc/cargo 1.97.0。

## 依存クレート（cliユニット、`unit-of-work-dependency.md`と整合）

| クレート | 種別 | 用途 | 選定理由 |
|---|---|---|---|
| `clap` | 通常依存 | `CliArgs`の引数解析 | derive API（`#[derive(Parser)]`）を採用（Q2=A）。Rustエコシステムで最も広く使われるCLI引数解析クレートで、ドキュメント・実績が豊富 |
| `serde_json` | 通常依存 | JSONデータのパース（`DataLoader`） | Rustエコシステム標準のJSON実装。`serde_json::Value`から`Value::from_serialize`でcore-engineの`Value`へ変換する |
| `serde_norway` | 通常依存 | YAMLデータのパース（`DataLoader`） | `serde`エコシステムと親和性が高いYAML実装。`serde_json`と同様の変換パターンで扱える。当初`serde_yaml`を想定していたが、Code Generation Step1でのビルド時に作者による非推奨化（deprecated）が判明したため、`serde_yaml` 0.9系とAPI互換のメンテナンス継続中の後継クレート`serde_norway`に変更した（詳細は下記「実装時の追加補正」参照） |
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

- `serde_yaml`以外のYAML実装（`yaml-rust`等）: `serde`エコシステムとの統合が`Value::from_serialize`の設計と直接整合するため、serdeベースの実装を採用（最終的に`serde_norway`）
- `clap`のbuilder API: derive APIを採用（Q2=A、理由は`nfr-requirements.md`参照）
- リリースビルド最適化設定: Q3=Aにより見送り

## 実装時の追加補正（要記録）: `serde_yaml`から`serde_norway`への変更

Code Generation Step1で`cargo build`を実行したところ、`serde_yaml v0.9.34+deprecated`と表示され、作者（dtolnay）が2024年にこのクレートを非推奨化していたことが判明した（NFR Requirements Q4決定時点では未確認だった）。新規プロジェクトの依存先として非推奨クレートを採用するのは長期的な保守性リスクがあるため、ユーザーに確認のうえ、`serde_yaml` 0.9系とAPI互換のドロップイン後継クレートである`serde_norway`（`serde_yaml`のフォーク、2024年12月時点でも更新継続中）に変更した。`Value::from_serialize`を経由する変換パターンは`serde_yaml`と同一のAPI形状（`serde_norway::Value`, `serde_norway::from_str`等）のため、`business-rules.md`/`domain-entities.md`の設計判断（BR-3.1〜3.4、`DataLoader`のシグネチャ）に変更は生じない。
