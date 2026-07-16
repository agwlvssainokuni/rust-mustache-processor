# Code Generation Plan — release-automation

## ユニットコンテキスト

- **対象**: GitHub Actionsによるクロスコンパイル・自動リリースワークフロー
- **参照**: `aidlc-docs/inception/requirements/release-automation-requirements.md`（FR-1〜FR-8, NFR-1〜NFR-3）、`aidlc-docs/construction/release-automation/infrastructure-design/infrastructure-design.md`
- **依存関係**: core-engine/cliユニットのコード（`Cargo.toml`のバイナリ名`mustache`、パッケージversion）に依存。既存ユニットへの変更はなし
- **成果物配置**:
  - アプリケーションコード（ワークフロー定義）: ワークスペースルート `.github/workflows/release.yml`
  - ドキュメント: `aidlc-docs/construction/release-automation/code/summary.md`

## ステップ一覧

- [x] **Step 1: ワークフロースケルトン作成** — `.github/workflows/release.yml`を新規作成し、`name`、`on`（タグpush `v*.*.*` ＋ `workflow_dispatch`）、`permissions: contents: write`、ジョブ名の骨組み（`verify-version`, `test`, `build`, `release`）を記述する（FR-1, FR-2対応）

- [x] **Step 2: verify-versionジョブ実装** — タグ名（`github.ref_name`）から`vX.Y.Z`の`X.Y.Z`部分を抽出し、`Cargo.toml`の`version`フィールドと比較。不一致なら`exit 1`で失敗させる（FR-4対応）

- [x] **Step 3: testジョブ実装** — `verify-version`に依存し、`ubuntu-latest`上で`dtolnay/rust-toolchain@stable`をセットアップして`cargo test`を実行（FR-5対応）

- [x] **Step 4: buildジョブ実装（マトリクス）** — `test`に依存し、4ターゲット（`x86_64-unknown-linux-gnu`/`ubuntu-latest`, `x86_64-apple-darwin`/`macos-13`, `aarch64-apple-darwin`/`macos-14`, `x86_64-pc-windows-msvc`/`windows-latest`）のマトリクスで`cargo build --release --target <triple>`を実行し、`mustache-<version>-<target-triple>.tar.gz`（Windowsのみ`.zip`）にアーカイブして`actions/upload-artifact`で保存（FR-3, FR-6対応）

- [x] **Step 5: releaseジョブ実装** — `build`に依存し、`actions/download-artifact`で全アーカイブを収集後、`softprops/action-gh-release`で`generate_release_notes: true`を指定してリリースを作成し、全アーカイブをアセットとして添付（FR-7, FR-8対応）

- [ ] **Step 6: README更新** — README.md/README.en.mdの「インストール」節に、GitHub Releasesからのビルド済みバイナリダウンロードによるインストール手順を追記（`cargo install --path .`と並記）

- [ ] **Step 7: ドキュメントサマリー作成** — `aidlc-docs/construction/release-automation/code/summary.md`に生成物一覧・設計判断の要約・既知の対象外事項（crates.io公開、CHANGELOG.md運用、Linux aarch64、日常CI）を記録

## 完了基準

- 全7ステップが完了し、`.github/workflows/release.yml`が単体で構文的に妥当なYAMLであること（`yamllint`または`python -c "import yaml; yaml.safe_load(...)"`等で検証）
- README.md/README.en.mdへの追記が完了していること
- ドキュメントサマリーが生成されていること

このプランがrelease-automationユニットのCode Generationにおける単一の情報源（single source of truth）である。
