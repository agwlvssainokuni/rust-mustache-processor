# Performance Test Instructions — rust-mustache-processor

## 方針

NFR Requirements（core-engine Q1, cli Q3）の決定により、本プロジェクトでは専用のベンチマーク・性能測定の自動テストスイートを構築していない。理由:

- core-engine: `render`はストリーミングAPIを持たず`String`を返すのみ（core-engine NFR Requirements Q1=A）。想定用途（CLIツールでの小〜中規模テキスト処理）に対して、現時点で性能要件・数値目標が要件定義（`requirements.md`）に存在しない
- cli: リリースビルドの追加最適化（LTO、strip等）も見送り済み（cli NFR Requirements Q3=A）。同様に具体的な性能目標が存在しない
- 上記はいずれもYAGNI判断であり、将来具体的な性能要件（例: 特定サイズのテンプレートを一定時間内に処理する等）が明確になった場合に、この方針を見直すことを想定している

## 実装済みの性能配慮

自動ベンチマークはないが、Code Generationの過程で以下の性能配慮を実装済み（`nfr-design-patterns.md`各パターン参照）:

| 配慮事項 | 内容 | 参照 |
|---|---|---|
| 出力バッファの事前確保 | `Mustache::render`が`String::with_capacity(template.source_len)`でテンプレート長を初期容量とする | core-engine NFR Design パターン4 |
| ネスト深度ガード | `MAX_NESTING_DEPTH = 100`により、パーシャル無限再帰やスタックオーバーフローを実測ベースで防止 | core-engine NFR Design パターン1、Code Generation Step4 |
| 出力の一括書き出し | cliは全テンプレートのレンダリング結果をメモリ上でバッファし、成功時に1回だけ書き出す（Atomic Output Buffering） | cli NFR Design パターン2 |

## 手動での性能確認（任意）

自動テストは実装していないが、大きめのテンプレート・データに対する処理時間を手動で確認したい場合は以下のように計測できる:

```bash
cargo build --release

# 大きめのテンプレート・データを用意した上で
time ./target/release/mustache large_template.tmpl --data large_data.json --output /tmp/out.txt
```

明確な性能劣化（体感で数秒以上かかる等）が観測された場合は、`nfr-requirements.md`（core-engine/cli）を見直し、性能要件の明確化とベンチマークスイート（`criterion`クレート等）の導入を検討することを推奨する。
