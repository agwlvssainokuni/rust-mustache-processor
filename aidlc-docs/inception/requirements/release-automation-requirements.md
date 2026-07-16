# Requirements — release-automation

## Intent Analysis

- **User Request**: "GitHub Actionsでクロスコンパイル・自動リリースの設定をお願いできますか？"
- **Request Type**: New Feature（既存のcore-engine/cliユニットへの機能追加ではなく、リポジトリ運用のための新規CI/CDインフラを追加する）
- **Scope Estimate**: Single Component（`.github/workflows/`配下に新規ワークフローファイルを追加する、リポジトリ横断のインフラ変更）
- **Complexity Estimate**: Simple〜Moderate（クロスコンパイル対象OSが複数あるため設定の作り込みは必要だが、業務ロジックの複雑さはない）

このユニットは、既存の`requirements.md`（core-engine/cli向け）が「シングルバイナリとしてクロスプラットフォーム配布できることを想定する（`cargo install`、GitHub Releases等での配布）」（NFR的な言及）と述べていた配布方針を、具体的なCI/CD自動化として実現するものである。CI/CD自動化そのものの要件定義は既存requirements.mdでは行われていなかったため、新規ユニットとして軽量なRequirements Analysisを実施した。

## 機能要件（Functional Requirements）

- **FR-1**: `v*.*.*`形式のGitタグがpushされたとき、GitHub Actionsワークフローが自動的に起動すること
- **FR-2**: GitHub Actions上で手動起動（`workflow_dispatch`）でも同じワークフローを実行できること
- **FR-3**: 以下のターゲットに対してクロスコンパイルし、それぞれの実行バイナリを生成すること
  - Linux x86_64（`x86_64-unknown-linux-gnu`）
  - macOS aarch64（`aarch64-apple-darwin`）
  - Windows x86_64（`x86_64-pc-windows-msvc`）
  - ~~macOS x86_64（`x86_64-apple-darwin`）~~ — 実装時の追加補正により対象外（後述）
- **FR-4**: ワークフロー実行時、`Cargo.toml`の`version`フィールドとpushされたGitタグ名が一致することを検証し、不一致の場合はリリースを作成せず失敗させること
- **FR-5**: リリース対象タグに対して、`cargo test`を実行し、失敗した場合はリリースを作成しないこと（テストゲート）
- **FR-6**: 各ターゲットのビルド成果物を、`mustache-<version>-<target-triple>.tar.gz`（Windowsターゲットのみ`.zip`）の命名規則でアーカイブすること
- **FR-7**: 生成した全アーカイブをGitHub Releaseのアセットとして添付し、リリースを作成すること
- **FR-8**: リリースノートはGitHubの自動生成機能（`generate_release_notes`）を用いて、直前タグからのPRタイトル一覧等から自動生成すること

## 非機能要件（Non-Functional Requirements）

- **NFR-1（保守性）**: ワークフロー定義は単一の`.github/workflows/release.yml`にまとめ、対象プラットフォームの追加・削除が容易な構造（マトリクスビルド）とすること
- **NFR-2（一貫性）**: バージョン番号の情報源は`Cargo.toml`の`version`のみとし、GitHub Release・アセットファイル名・タグの間で不整合が生じないこと（FR-4のタグ整合性検証で担保）
- **NFR-3（安全性）**: テストが1件でも失敗した状態のコミットに対してリリースが作成されないこと（FR-5で担保）

## スコープ外（Out of Scope）

- crates.ioへの`cargo publish`自動化（別途、必要になった時点で改めて要件定義を行う）
- `CHANGELOG.md`等の変更履歴ファイルの手動運用（当面はGitHub自動生成のリリースノートで代替する）
- Linux aarch64ターゲットのクロスコンパイル（需要が確認された時点で追加を検討する）
- PRやpush時に毎回実行する「テスト用CI」ワークフロー（本ユニットはリリース時のワークフローのみを対象とする。日常的なCIの要否は別途判断する）
- **macOS x86_64（Intel）ターゲット**（実装時の追加補正、後述）

## 実装時の追加補正（要記録）

- **macOS x86_64（`x86_64-apple-darwin`）ターゲットを対象外に変更**: `v0.1.0`の初回リリース実行で、`macos-13`（Intel）ランナーがキューから進行せず、長時間経過後にジョブがcancelled状態になる事象を確認。GitHub-hostedランナーのIntel macOS環境の提供状況変化（Apple Siliconへの移行に伴う容量縮小等）が疑われる。ユーザーの指示によりmacOS x86_64ターゲットをビルドマトリクスから削除し、対象プラットフォームをLinux x86_64・macOS aarch64・Windows x86_64の3種に変更した。将来的にmacOS x86_64ランナーの状況が改善すれば再追加を検討する。

## Summary of Key Decisions

| 項目 | 決定内容 |
|---|---|
| トリガー | タグpush（`v*.*.*`）＋手動起動の両方 |
| 対象プラットフォーム | Linux x86_64 / macOS aarch64 / Windows x86_64（macOS x86_64は`macos-13`ランナーの実行不安定のため対象外） |
| バージョン管理 | `Cargo.toml`の`version`を正とし、タグ名との一致をCIで検証 |
| リリースノート | GitHub自動生成機能を利用 |
| アーカイブ命名 | `mustache-<version>-<target-triple>.tar.gz`／`.zip`（Windows） |
| テストゲート | リリース前に`cargo test`を実行し、失敗時はリリースしない |
| crates.io公開 | 対象外（別ユニットとして将来検討） |

各決定の詳細な理由は`release-automation-requirement-verification-questions.md`を参照。
