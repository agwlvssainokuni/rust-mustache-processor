# Build and Test Summary — rust-mustache-processor

## 概要

core-engine（`mustache_processor`ライブラリ）・cli（`mustache`バイナリ）両ユニットのCode Generation完了後、ビルド・テストの最終確認を実施した。v0.2.0（Mustacheオプションモジュール フルサポート）でのCode Generation完了後、再度ビルド・テストの最終確認を実施した（本節末尾「v0.2.0 追加確認」参照）。

## ビルド結果

```bash
cargo build
```

- ライブラリ（`mustache_processor`）・バイナリ（`mustache`）ともに**警告0件**でビルド成功
- 詳細は`build-instructions.md`参照

## テスト結果

```bash
cargo test
```

| テストグループ | 場所 | 件数 | 結果 |
|---|---|---|---|
| core-engineユニットテスト | `src/lib.rs`以下 | 72件 | 全成功 |
| cliユニットテスト | `src/main.rs`以下 | 33件（proptest 3件含む） | 全成功 |
| core-engineプロパティベーステスト | `tests/proptest/` | 7件 | 全成功 |
| core-engine spec conformanceテスト | `tests/spec/` | 6モジュール136ケース | 全成功 |
| doctest | `src/lib.rs`クレートdoc | 1件 | 全成功 |

**合計: 119テスト実行単位、全て成功**（`tests/spec/`の136ケースは6つのテスト関数に集約されているため、実行単位としては119）。

詳細は`unit-test-instructions.md`・`integration-test-instructions.md`参照。

## 品質保証の観点別まとめ

| 観点 | 実施内容 |
|---|---|
| 単体テスト | 105件（core-engine 72件 + cli 33件） |
| プロパティベーステスト | 10件（core-engine 7件 + cli 3件、`requirements.md` NFR-3のPBT拡張機能フル適用） |
| 公式spec準拠 | mustache/spec必須6モジュール136ケース100%成功（`requirements.md` NFR-2） |
| ユニット間結合 | cliの`run_inner`系テスト7件がcore-engine公開APIとの結合を実地検証 |
| ドキュメント | core-engineは`#![deny(missing_docs)]`で全公開API文書化を強制、doctestで使用例を検証 |
| 手動動作確認 | `cargo run --bin mustache`による実ファイル・実データでのエンドツーエンド動作確認済み |
| 性能 | 専用ベンチマークは未実装（要件上の数値目標なし、YAGNI判断）。事前確保・深度ガード等の配慮は実装済み |

## Code Generation全体を通じて発見・修正した主要な設計補正

両ユニットのCode Generationを通じて、承認済みの上位ステージ（Application Design/Functional Design）の決定を、実装・検証段階で発見した事実に基づき見直した箇所がある。詳細は各ユニットの`summary.md`を参照:

- **core-engine**（`aidlc-docs/construction/core-engine/code/summary.md`）: Value/Mapの整合性補正、真偽判定ルールの補正、ネスト深度上限の実測に基づく修正、公式spec conformanceテストで判明した7件の重大な補正（暗黙のイテレータ、ドット区切り名前、スタンドアロン判定の行単位化、パーシャル未解決時の既定動作、パーシャル循環検出の削除等）
- **cli**（`aidlc-docs/construction/cli/code/summary.md`）: `serde_yaml`の非推奨化発見による`serde_norway`への変更、循環モジュール依存の回避、PBTテスト実装場所の補正

いずれも、公式仕様・実測・言語仕様上の制約という客観的根拠に基づく補正であり、対応する設計文書（`business-rules.md`, `tech-stack-decisions.md`等）に経緯を記録済み。

## 既知の対象外事項

- ストリーミング出力API（core-engine NFR Requirements Q1）
- リリースビルドの追加最適化（cli NFR Requirements Q3）
- CLIバイナリのサブプロセス起動によるend-to-endテスト（`integration-test-instructions.md`参照、YAGNI）
- 専用の性能ベンチマークスイート（`performance-test-instructions.md`参照）
- ブロックの再インデント処理（`~inheritance`の一部、BR-10.7。下記「v0.2.0 追加確認」参照）

## v0.2.0 追加確認（Mustacheオプションモジュール フルサポート）

`mustache-optional-modules-requirements.md`・`core-engine-mustache-optional-modules-code-generation-plan.md`に基づくcore-engine拡張（Code Generation全13ステップ）完了後、ビルド・テストを再確認した。

### ビルド結果

```bash
cargo build
```

- ライブラリ・バイナリともに**警告0件**でビルド成功

### テスト結果

```bash
cargo test
```

| テストグループ | 件数 | 結果 |
|---|---|---|
| core-engineユニットテスト | 84件（v0.1.1時点72件+12件） | 全成功 |
| cliユニットテスト | 33件 | 全成功（変更なし） |
| core-engineプロパティベーステスト | 9件（v0.1.1時点7件+2件） | 全成功 |
| core-engine spec conformanceテスト | 必須6モジュール136ケース + `~lambdas`10ケース + `~dynamic-names`21ケース + `~inheritance`23/27ケース | 全成功（`~inheritance`は既知の制限4件を`#[ignore]`で除外） |
| doctest | 1件 | 全成功 |

**合計: 137テスト実行単位、既定の`cargo test`で全て成功**（`~inheritance`の既知の制限4件は`inheritance_known_limitations`として`#[ignore]`属性を付与し、`cargo test --test spec -- --ignored`で個別に確認可能）。

### Build and Testで発見した重大な問題と対応

当初、`~inheritance`の既知の制限（BR-10.7）4件を1つのテスト関数内に含めていたため、既定の`cargo test`がテスト失敗として終了する状態だった。これは`release-automation-requirements.md` FR-5（`cargo test`失敗時はリリースを作成しないテストゲート）を直撃し、**v0.2.0が永久にリリースできなくなる**重大な問題だった。

ユーザー確認のうえ、`tests/spec/conformance.rs`のinheritanceテストを以下の2つに分割して解消した:
- `inheritance`（既定で実行、23件の合格ケースのみ対象）
- `inheritance_known_limitations`（`#[ignore]`属性、既知の制限4件のみ対象。理由を属性に明記し、`--ignored`指定時のみ実行される）

この対応により、既定の`cargo test`はクリーンに成功し、release-automationのテストゲートを正常に通過できることを確認した。

### v0.2.0での既知の対象外事項

- ブロックの再インデント処理（`~inheritance`のうち4ケース、BR-10.7）。フォローアップ課題として別途対応予定

## 次のステップ

Build and Testステージの成果物確認後、v0.2.0としてリリースする（release-automationワークフローによる自動リリース）。
