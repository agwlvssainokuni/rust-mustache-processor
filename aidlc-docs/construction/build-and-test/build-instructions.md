# Build Instructions — rust-mustache-processor

## 前提条件

- Rust（edition 2024対応）、rustc/cargo 1.97.0以降（開発環境で動作確認済み、`requirements.md` Technical Context）
- 追加のシステム依存なし（純Rustクレートのみで構成）

## ビルド対象

本リポジトリは単一のCargoパッケージ（`rust-mustache-processor`）に、2つのビルドターゲットを持つ:

| ターゲット種別 | 名前 | パス | 内容 |
|---|---|---|---|
| ライブラリ | `mustache_processor` | `src/lib.rs` | core-engineユニット（Mustacheパース・レンダリングエンジン） |
| バイナリ | `mustache` | `src/main.rs` | cliユニット（CLIラッパー） |

## ビルドコマンド

### 開発ビルド（デバッグ）

```bash
# 全ターゲット（ライブラリ+バイナリ）をビルド
cargo build

# バイナリのみビルド
cargo build --bin mustache

# ライブラリのみビルド
cargo build --lib
```

### リリースビルド

```bash
cargo build --release
```

NFR Requirements（cli）Q3の決定により、`[profile.release]`の追加最適化設定（LTO、strip等）は行っていない。Cargoの標準的なリリースプロファイルに従う。

### ドキュメント生成

```bash
# core-engineライブラリのAPIドキュメントを生成（#![deny(missing_docs)]により全公開項目が文書化済み）
cargo doc --no-deps --open
```

cliバイナリは公開APIを持たないため、`cargo doc`の対象外（NFR Requirements cli Q1の決定）。

## ビルド成果物の配置

```
target/debug/mustache      # 開発ビルドのCLIバイナリ
target/release/mustache    # リリースビルドのCLIバイナリ
```

`cargo install --path .`により、`mustache`バイナリを`~/.cargo/bin/`等へインストールできる（NFR-1: シングルバイナリでのクロスプラットフォーム配布）。

## ビルド警告の扱い

Code Generation完了時点（全ステップ）で、`cargo build`はワークスペース全体で警告0件であることを確認済み（`core-engine/code/summary.md`, `cli/code/summary.md`参照）。新たな警告が出た場合は、リリース前に解消することを推奨する。
