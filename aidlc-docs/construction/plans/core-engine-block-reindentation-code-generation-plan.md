# Code Generation Plan — core-engine: BR-10.7 ブロック再インデント処理

## ユニットコンテキスト

- **対象ユニット**: core-engine（既存ユニットの拡張）
- **ワークスペースルート**: ~/Documents/project/git/rust-mustache-processor
- **依存関係**: cliユニットへの影響なし（`Node`/`RenderState`はいずれも非公開。公開APIの変更なし）
- **参照する成果物**: `aidlc-docs/construction/core-engine/functional-design/business-rules.md`（BR-10.7〜BR-10.13）、`domain-entities.md`（`Node::Block`拡張）、`business-logic-model.md`（処理フロー）

## 実装方針の要点（Functional Designより）

- `Node::Block`にBR-10.8のclearance情報（`open_clears_start`/`open_clears_end`/`close_clears_start`/`close_clears_end`）・`raw`・`open_indent`をパース時に記録する。判定は既存のPass2（行単位スタンドアロン判定）とは別に、タグの生バイトオフセットに対する直接の文字列検査で行う（Pass2の行単位判定は複数タグ混在行を「行全体」で判定するのに対し、BR-10.8のclearanceはタグ単体の前後を厳密に判定する必要があり、意味が異なるため）
- `RenderState.block_overrides`を`Vec<HashMap<String, Node>>`に変更し、レンダリング時に`Node::Block`へ遭遇するたびにオーバーライドの有無をその場で判定する（事前の木一括置換`substitute_blocks`を廃止）
- Nested block reindentation（Case D）はアルゴリズム的に複雑なため、実装後に`cargo test`で実地検証し、不一致があれば都度補正する（Functional Design Q3で承認済みの方針）

## Code Generation ステップ

- [ ] Step 1: `src/ast.rs` — `Node::Block`にフィールド追加（`raw: String`, `open_clears_start: bool`, `open_clears_end: bool`, `close_clears_start: bool`, `close_clears_end: bool`, `open_indent: String`）
- [ ] Step 2: `src/parser.rs` — `Frame::Block`に`open_start`/`open_end`を追加。`clears_at_start`/`clears_at_end`/`leading_whitespace_before`ヘルパー関数を追加。`Node::Block`構築時（`SectionEnd`処理）に新フィールドを算出して設定
- [ ] Step 3: `src/parser.rs` — Step 2の追加に対する単体テストを追加（`clears_at_start`/`clears_at_end`の境界ケース、`\r\n`、`open_indent`、`raw`の直接検証）
- [ ] Step 4: `src/renderer.rs` — `RenderState.block_overrides`の型を`Vec<HashMap<String, Node>>`に変更。`build_block_overrides`を全体の`Node`を保持するよう変更
- [ ] Step 5: `src/renderer.rs` — `substitute_blocks`/`substitute_blocks_node`を削除。`render_parent`を簡素化（事前一括置換を廃止し、`parent_nodes`をそのまま`render_nodes`に渡す）
- [ ] Step 6: `src/renderer.rs` — `render_nodes`の`Node::Block`処理を`render_block`関数に置き換え。`find_effective_override`（外側優先の実効オーバーライド検索）・`dedent_block`（BR-10.10）・`expansion_indent_for`（BR-10.11）・`leading_whitespace_of_first_line`・`strip_prefix_from_lines`ヘルパーを追加。`enter_depth`によるネスト深度ガード（BR-10.13）を適用
- [ ] Step 7: `src/renderer.rs` — Step 4〜6の追加に対する単体テストを追加（主要な再インデントパターンの直接検証: 定義箇所インデント除去、展開箇所インデント付与、末尾改行の強制付与規則）
- [ ] Step 8: `tests/spec/conformance.rs` — `inheritance_known_limitations`関数と`INHERITANCE_KNOWN_LIMITATIONS`定数を削除し、4ケースを`inheritance()`本体（`run_module("~inheritance")`、全27ケース）に統合
- [ ] Step 9: `cargo test --test spec`実行によりspec conformance（`~inheritance`27/27を含む）を確認。不一致が見つかった場合はBR-10.7〜BR-10.12のアルゴリズム・実装を補正し、再実行する（収束するまで繰り返す）
- [ ] Step 10: `cargo test`（ワークスペース全体）・`cargo clippy`・`cargo fmt --check`を実行し、既存テスト（ユニット84・proptest9・doctest等）に回帰がないことを確認
- [ ] Step 11: `Cargo.toml`のバージョンを0.2.0から0.2.1に更新（パッチリリース、Functional Design Q5）
- [ ] Step 12: ドキュメント更新 — `mustache-optional-modules-requirements.md`（既知の制限セクションを削除し27/27に更新）、`README.md`/`README.en.md`（既知の制限の記述を削除、spec準拠率更新）、`unit-test-instructions.md`/`integration-test-instructions.md`（テスト件数・`#[ignore]`記述の更新）
- [ ] Step 13: `aidlc-docs/construction/core-engine/code/summary.md`に本拡張の生成物一覧・テスト結果・spec準拠状況を追記。`aidlc-state.md`を更新

## コミット方針

既存のCode Generationと同様、各ステップ完了ごとにコミットする（`feedback_auto_commit_after_audit.md`の方針に準じ、audit.md更新と合わせてこまめにコミット）。
