# NFR Design Patterns — core-engine

`nfr-requirements.md`の決定事項と`core-engine-nfr-design-plan.md`のQ1〜Q3を、具体的な設計パターンとして落とし込む。

## 適用対象外のパターンカテゴリ

| カテゴリ | 判定 | 理由 |
|---|---|---|
| Resilience Patterns（リトライ、サーキットブレーカー等） | N/A | 外部サービス呼び出し・ネットワークI/Oを伴わない純粋な変換処理のため |
| Scalability Patterns（負荷分散、水平スケール等） | N/A | 単一プロセスのライブラリであり、スケーリングの概念が適用されないため |
| Security Patterns（認証・認可・入力サニタイズ等） | N/A | Security Baseline拡張機能が無効のため |

## 適用パターン

### パターン1: Recursion Guard（再帰防御）
- **目的**: セクションの入れ子・パーシャルの再帰呼び出しによるスタックオーバーフローを防ぐ（NFR Requirements Q2、NFR Design Q2=A）
- **実装方針**: `RenderState`（論理コンポーネント参照）に単一の`depth: usize`カウンタを持たせ、セクション・パーシャルいずれの再帰呼び出しでも呼び出し前にインクリメント、呼び出し後にデクリメントする。`depth`が定数`MAX_NESTING_DEPTH`（1000）を超えた時点で`RenderErrorKind`の新バリアント（例: `MaxNestingDepthExceeded`）を返し、再帰を打ち切る

### パターン2: Cycle Detection（循環検出）
- **目的**: パーシャルの直接的・間接的な自己参照を検出し、無限再帰を防ぐ（Functional Design Q4=B）
- **実装方針**: `RenderState`に解決中のパーシャル名チェーン`partial_chain: Vec<String>`を持たせる。パーシャル解決前にチェーン内の重複を確認し、重複があれば`RenderErrorKind::PartialCycleDetected`を返す。解決成功時はチェーンにpushし、そのパーシャルのレンダリング完了後にpopする（スコープの外側では循環とみなさない、兄弟パーシャルの多重参照は許容）

### パターン3: Bundled Render State（レンダリング状態の集約）
- **目的**: コンテキストスタック・ネスト深度・パーシャルチェーン・strictフラグという複数の横断的関心事を、再帰呼び出しのシグネチャを汚さずに管理する（NFR Design Q1=A）
- **実装方針**: `RenderState`構造体（論理コンポーネント参照）にまとめ、`Renderer`の内部再帰関数は`&mut RenderState`を1つ受け取るのみとする

### パターン4: Capacity Pre-allocation（出力バッファの事前確保）
- **目的**: レンダリング結果`String`構築時の再アロケーション回数を削減する（NFR Design Q3=A）
- **実装方針**: `Mustache::render`のエントリポイントで`String::with_capacity(template_source_len)`により初期容量を確保してから`Renderer`に渡す。テンプレートの元の文字列長（`Template`が保持する、または`render`呼び出し時にParserの入力長を記録）を初期値とする

### パターン5: Compile-time Documentation Enforcement（コンパイル時ドキュメント強制）
- **目的**: 公開APIのrustdoc欠落を防ぐ（NFR Requirements Q3=B）
- **実装方針**: `lib.rs`クレートルートに`#![deny(missing_docs)]`を付与する。Code Generation時に全`pub`項目へのドキュメントコメント記述と併せて適用する

### パターン6: Tuned Property Test Configuration（プロパティテスト設定の調整）
- **目的**: PBTの実行時間をCIで実用的な範囲に保つ（NFR Requirements Q4=B）
- **実装方針**: `tests/proptest/`配下の各テストモジュールで、軽量なプロパティ（エスケープ往復、セクション相補性等）は`ProptestConfig`のデフォルト（256ケース）を使用し、構造化テンプレート生成を伴う重いプロパティ（Parserの入れ子構造保存、パーシャル循環検出）には`ProptestConfig { cases: 64, ..Default::default() }`相当の個別設定を適用する
