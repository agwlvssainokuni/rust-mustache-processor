# Unit of Work Plan — rust-mustache-processor

`application-design.md`で定義したcore-engine（ライブラリ）/cli（バイナリ`mustache`）の2ユニット構成を前提に、Units Generationを実施する。

**注記（User Stories省略への対応）**: 本プロジェクトはUser Storiesステージを省略している（単一開発者向けツールで複数ペルソナなし）。そのため`unit-of-work-story-map.md`は「ストーリー→ユニット」ではなく「要件（FR-1〜FR-8）→ユニット」のマッピングとして生成する。

## Plan Checklist

### Part 1 — Planning
- [x] Step 1-2: ユニット計画・必須成果物の洗い出し（本ファイル作成）
- [x] Step 3-4: 決定が必要な論点を洗い出し、質問として本ファイルに埋め込み
- [ ] Step 5-6: ユーザーからの回答収集
- [ ] Step 7-8: 回答の曖昧さ分析・フォローアップ（必要な場合）
- [ ] Step 9: 承認依頼
- [ ] Step 10-11: 承認ログ記録・状態更新

### Part 2 — Generation
- [ ] `aidlc-docs/inception/application-design/unit-of-work.md` 生成（ユニット定義・コード構成方針を含む）
- [ ] `aidlc-docs/inception/application-design/unit-of-work-dependency.md` 生成（依存関係マトリクス）
- [ ] `aidlc-docs/inception/application-design/unit-of-work-story-map.md` 生成（要件→ユニットのマッピング）
- [ ] ユニット境界・依存関係の妥当性検証
- [ ] 全要件（FR-1〜FR-8）がいずれかのユニットに割り当てられていることを確認
- [ ] 完了メッセージ提示・ユーザー承認待ち

## 検討済み・決定済みの論点（Application Designより引き継ぎ）

以下はApplication Designで既に確定しており、Units Generationで再確認不要:
- ユニット構成: core-engine（ライブラリ）、cli（バイナリ`mustache`）の2ユニット
- ユニット間依存: 単方向（cli → core-engine）、`component-dependency.md`に記載済み
- コンポーネントのユニット割当: `components.md`に記載済み
- チーム体制: 単一開発者のため「チーム間の分割」の観点は適用外

## 決定が必要な論点（質問）

### Question 1: Cargoプロジェクトの物理構成
現在の`Cargo.toml`は要件定義前のアドホックな初期化により、単一パッケージ（`rust-mustache-processor`）に`[[bin]] name = "mustache", path = "src/main.rs"`を追加した構成になっている。Application Designで定義したcore-engine/cliの2ユニットを、実際のCargoプロジェクト構造にどう対応させるか。

A) 単一クレート内でライブラリ+バイナリを分離（`src/lib.rs`がcore-engine、`src/main.rs`がcli）。現在のCargo.tomlをベースに、`src/lib.rs`を追加する形で拡張。パッケージは1つのまま

B) Cargo workspaceに変更し、`crates/core-engine`と`crates/cli`を独立クレートとして分割。ルートに`Cargo.toml`（workspaceマニフェスト）を配置

C) Other（[Answer]: タグの下に詳細を記載）

[Answer]: 

### Question 2: テストコード・spec conformanceテストデータの配置
NFR-2で要求される公式mustache/specコンフォーマンステストスイート（JSONテストケース集）およびPBT（proptest）テストコードの配置場所。

A) core-engineユニット側（`src/lib.rs`と同じクレート）の`tests/`ディレクトリに配置。cliは薄いラッパーのためユニットテストのみで十分とする

B) ワークスペース直下の共通`tests/`ディレクトリに配置し、core-engine/cli両方から参照可能にする

C) Other（[Answer]: タグの下に詳細を記載）

[Answer]: 