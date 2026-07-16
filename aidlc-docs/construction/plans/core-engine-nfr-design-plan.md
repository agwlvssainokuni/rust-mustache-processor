# NFR Design Plan — core-engine

`nfr-requirements.md`（core-engine）の決定事項を、具体的な設計パターン・論理コンポーネントに落とし込む。

## Plan Checklist

- [x] Step 1: NFR Requirements成果物の分析
- [x] Step 2-4: 計画作成・質問洗い出し（本ファイル）
- [x] Step 5: ユーザー回答収集・曖昧さ分析（Q1=A, Q2=A, Q3=A、いずれも推奨通り。曖昧・矛盾なし）
- [x] Step 6: NFR設計成果物生成
  - [x] `nfr-design-patterns.md`
  - [x] `logical-components.md`
- [ ] Step 7-9: 完了メッセージ提示・承認待ち・記録

## カテゴリ別評価

| カテゴリ | 適用可否 | 理由 |
|---|---|---|
| Resilience Patterns | N/A | 単一プロセス内の純粋な変換処理（テンプレート+データ→文字列）であり、外部サービス呼び出し・ネットワークI/Oを伴わない。リトライ・サーキットブレーカー等の耐障害性パターンは適用対象がない（Resiliency Baseline無効、`nfr-requirements.md`） |
| Scalability Patterns | N/A | 水平/垂直スケーリングの概念が適用されないライブラリのため（`nfr-requirements.md`） |
| Security Patterns | N/A | Security Baseline拡張機能が無効（`nfr-requirements.md`） |
| Performance Patterns | 適用あり | Question 3参照（出力バッファの事前確保方針） |
| Logical Components | 適用あり | Question 1・2参照（レンダリング内部状態の設計、ネスト深度制限の実装パターン） |

## 決定が必要な論点（質問）

### Question 1: レンダリング内部状態のまとめ方
NFR Requirements Q2（ネスト深度制限）・Functional Design Q4（パーシャル循環検出）により、Rendererの再帰呼び出しには「コンテキストスタック」「現在のネスト深度」「解決中のパーシャル名チェーン」「strictフラグ」の複数の状態を引き回す必要がある。これらをどう設計するか。

A) 上記4つの状態を1つの内部構造体（例: `RenderState`）にまとめ、再帰呼び出しには`&mut RenderState`（または所有権を持つ形）を1つ渡すだけにする

B) 状態ごとに個別の引数として再帰呼び出しの関数シグネチャに並べる（例: `render_node(node, context_stack, depth, partial_chain, strict)`）

C) Other（[Answer]: タグの下に詳細を記載）

[Answer]: A

**理由**: 個別引数（B）だと、今後さらに状態が増えるたびに全ての再帰呼び出し箇所のシグネチャを変更する必要が生じる。構造体にまとめておけば変更影響がその構造体定義とアクセス箇所に閉じ、`Renderer`内部の再帰関数のシグネチャも`render_node(node, state: &mut RenderState)`のようにシンプルに保てる。あくまでcore-engine内部（非公開）の設計なので、公開APIの複雑化にはつながらない。

### Question 2: ネスト深度制限の具体的な設計
NFR Requirements Q2で「最大ネスト深度（例1000階層）」までは決定済み。具体的な制限方式をどうするか。

A) セクションの入れ子とパーシャルの再帰呼び出しを区別せず、単一のカウンタで合算してカウントする（実装がシンプル）。上限値は1000とする

B) セクションの入れ子とパーシャルの再帰呼び出しを別々のカウンタで管理し、それぞれに独立した上限値を設ける（例: セクション入れ子は500、パーシャル再帰は100）。ユースケースに応じて意味のある制限を個別にかけられるが実装がやや複雑になる

C) Other（[Answer]: タグの下に詳細を記載）

[Answer]: A

**理由**: セクション入れ子とパーシャル再帰を分けて管理すると境界ケース（例: セクション499階層+パーシャル再帰1回）の扱いが複雑になり、テスト（spec conformance/PBT）で検証すべき組み合わせも増える。本質的な懸念はRustのコールスタックを枯渇させないことであり、両者を区別する実用上の理由がない限り、合算した単一カウンタの方がシンプルで説明しやすい。上限値1000はNFR Requirementsの例示通りとする。

### Question 3: 出力バッファの事前確保方針
`Mustache::render`が返す`String`の構築時、パフォーマンス最適化としてテンプレートサイズに基づく容量の事前確保（`String::with_capacity`）を行うか。

A) 行う。テンプレート文字列の長さ（バイト数）をそのまま初期容量として`String::with_capacity(template.len())`する。再アロケーションを減らせる、実装コストもほぼゼロ

B) 行わない。`String::new()`から開始し、Rustの標準的な再アロケーション戦略に任せる。実装をシンプルに保つ

C) Other（[Answer]: タグの下に詳細を記載）

[Answer]: A

**理由**: `String::with_capacity(template.len())`は実装コストがほぼゼロでありながら再アロケーション回数を確実に減らせる。変数展開後の出力は概ねテンプレート長と同程度かそれ以上になることが多く、初期値としては妥当な下限見積もりになる。過大な予約にもならず、デメリットが小さい。
