# NFR Requirements Plan — cli

`cli-functional-design-plan.md`・`domain-entities.md`・`business-rules.md`を踏まえた、cliユニットの非機能要件の評価計画。

## カテゴリ別評価

| カテゴリ | 適用可否 | 理由 |
|---|---|---|
| Security Baseline | N/A | プロジェクト全体でExtension無効（`requirements.md` NFR-4、`aidlc-state.md`） |
| Resiliency Baseline | N/A | プロジェクト全体でExtension無効（`requirements.md` NFR-5、`aidlc-state.md`） |
| Scalability | N/A | 単一プロセスのローカルCLIツールであり、水平/垂直スケーリングの概念が適用されない |
| Performance | 適用あり | Question 3参照（リリースビルド最適化） |
| Reliability | 適用あり | エラー処理はFunctional Design（BR-7.x）で既に決定済み。追加論点なし |
| Maintainability | 適用あり | Question 1・2参照（missing_docs適用要否、clap API形式） |
| Usability | 適用あり | Functional Design（BR-1.x〜BR-7.x）で既に決定済み。追加論点なし |
| PBT（NFR-3） | 適用あり | Question 4参照 |

## Plan Checklist

- [ ] Step 1: Functional Design成果物の分析（本ファイル作成）
- [ ] Step 2-4: 未決定論点の洗い出し・質問提示（本ファイル）
- [ ] Step 5: ユーザー回答収集・曖昧さ分析
- [ ] Step 6: NFR Requirements成果物生成（nfr-requirements.md, tech-stack-decisions.md）
- [ ] Step 7-9: 完了メッセージ提示・承認待ち・記録

## 決定が必要な論点（質問）

### Question 1: cliバイナリへの`#![deny(missing_docs)]`適用要否

core-engineはNFR Requirements Q3=Bにより`#![deny(missing_docs)]`を採用した（ライブラリとして配布されるため）。cliはバイナリであり、外部の利用者が`cargo doc`でAPIを参照することは想定されない。

A) 適用しない。cliは公開APIを持たないバイナリであり、rustdocによる利用者向けドキュメントの必要性がcore-engineほど高くない。過剰な文書化コストを避ける（YAGNI）

B) 適用する。core-engineと一貫した品質基準を維持する

C) Other（[Answer]: タグの下に詳細を記載）

[Answer]:

### Question 2: clap APIの利用形式

`unit-of-work-dependency.md`で`clap`の採用は決定済みだが、derive APIとbuilder APIのどちらを使うかは未決定。

A) derive API（`#[derive(Parser)]`でCliArgs構造体に直接注釈する）を使う。宣言的で構造体定義と引数定義が一体化し保守しやすい

B) builder API（`Command::new(...).arg(...)`で実行時に構築する）を使う。より柔軟だが構造体との対応をコード側で手動管理する必要がある

C) Other（[Answer]: タグの下に詳細を記載）

[Answer]:

### Question 3: リリースビルドの最適化設定

NFR-1（シングルバイナリでのクロスプラットフォーム配布）を踏まえ、`Cargo.toml`に`[profile.release]`でバイナリサイズ・実行速度の最適化設定（LTO、strip、opt-level等）を追加するか。

A) 追加しない。Cargoのデフォルトrelease設定のまま変更しない。NFR-1は配布形態（シングルバイナリであること）を求めているのみで、サイズ・速度の具体的な最適化までは要求していない。過剰最適化を避け、必要になった時点で追加する（YAGNI）

B) 追加する。`lto = true`, `strip = true`, `opt-level = "z"`（サイズ優先）等を設定し、配布バイナリを最適化する

C) Other（[Answer]: タグの下に詳細を記載）

[Answer]:

### Question 4: PBT運用方針（cliユニット）

`business-logic-model.md`（cli）のTestable PropertiesはDataLoaderのJSON/YAML往復変換・形式判定の決定性の3件で、いずれもファイルI/Oを伴わない軽量なデータ変換。

A) core-engineと同じ`proptest`（既に`[dev-dependencies]`に追加済み）を再利用し、cliのプロパティは全て軽量なためデフォルト256ケースとする

B) cli独自に異なるPBTフレームワークを採用する

C) Other（[Answer]: タグの下に詳細を記載）

[Answer]:
