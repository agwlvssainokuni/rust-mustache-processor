# Unit of Work Story Map — rust-mustache-processor

**注記**: 本プロジェクトはUser Storiesステージを省略している（単一開発者向けツールで複数ペルソナなし、`aidlc-state.md`参照）。そのため本ドキュメントは「ストーリー→ユニット」ではなく、`requirements.md`の機能要件（FR-1〜FR-8）を対象に「要件→ユニット」のマッピングとして作成する。

## 要件 → ユニット マッピング

| 要件 | 概要 | 対応ユニット | 対応コンポーネント |
|---|---|---|---|
| FR-1 | Mustache処理エンジンの自作 | core-engine | Parser, Renderer |
| FR-2 | 提供形態（ライブラリ+CLI） | core-engine（ライブラリ本体）, cli（薄いラッパー） | Mustache（core-engine）, CliRunner（cli） |
| FR-3 | 入力データ形式（JSON/YAML） | cli | DataLoader |
| FR-4 | Mustache仕様準拠範囲（必須機能一式） | core-engine | Parser, Renderer |
| FR-5 | CLIの入出力インターフェース（ファイル/標準入出力） | cli | CliArgs, IoController |
| FR-6 | パーシャルの解決方法（ディレクトリベース） | core-engine（抽象化・解決ロジック）, cli（デフォルトディレクトリ決定） | PartialResolver, DirectoryPartialResolver（core-engine）, CliRunner（cli） |
| FR-7 | 未定義変数の挙動（既定継続/strictモード） | core-engine | Renderer, Mustache（`with_strict`） |
| FR-8 | 著作権・ライセンス表記 | core-engine, cli（両方の全ソースファイル） | 横断的要件。Code Generationステージで全ファイルに適用 |

## ユニット別の要件充足確認

### core-engine
- FR-1, FR-4, FR-7を主として担当し、FR-2（ライブラリ本体）とFR-6（パーシャル解決の抽象化）にも関与
- 全ての機能要件がコンパイラで強制される公開API境界内で実現可能であることを確認

### cli
- FR-3, FR-5を主として担当し、FR-2（薄いラッパー）とFR-6（デフォルトディレクトリ決定）にも関与
- core-engineの公開APIのみを利用して全ての要件を満たせることを確認（内部モジュールへの依存不要）

## 検証結果
- FR-1〜FR-8の全要件がいずれかのユニット（または両方）に割り当て済みであることを確認
- 割り当て漏れ・重複による責務の曖昧さは検出されなかった
