# Unit of Work Dependency — rust-mustache-processor

## ユニット間依存関係マトリクス

| ユニット | 依存先ユニット | 依存の種類 | 備考 |
|---|---|---|---|
| core-engine | （なし） | — | 他ユニットに一切依存しない。外部クレートは`serde`（Serializeトレイト境界）のみ |
| cli | core-engine | 関数呼び出し（プロセス内、同期） | core-engineの公開API（`Mustache`, `Template`, `Value`, `PartialResolver`, `DirectoryPartialResolver`, エラー型）のみを利用。`Parser`/`Renderer`など非公開モジュールには依存できない（コンパイラにより強制） |

**依存方向**: cli → core-engine の単方向のみ。逆方向（core-engine → cli）の依存は存在しない（`application-design.md`のアーキテクチャ原則を継承）。

## クレート内モジュール依存（参考: component-dependency.mdより詳細化）

### core-engineユニット内
| モジュール | 依存先 | 種類 |
|---|---|---|
| `value.rs` | （なし） | 純粋データ構造 |
| `parser.rs` | `value.rs`（一部） | 構文解析 |
| `renderer.rs` | `value.rs`, `parser.rs`, `partial.rs` | レンダリングロジック、パーシャル再帰解析 |
| `partial.rs` | （なし） | トレイト定義 + `std::fs`（`DirectoryPartialResolver`） |
| `error.rs` | （なし） | エラー型定義 |
| `lib.rs` | 上記全モジュール | 公開ファサード（`Mustache`エンジン）としてオーケストレーション |

### cliユニット内
| モジュール | 依存先 | 種類 |
|---|---|---|
| `cli/args.rs` | （なし） | argv解析（外部クレート`clap`等） |
| `cli/io.rs` | `cli/args.rs`, `std::fs`/`std::io` | ファイル・標準入出力アクセス |
| `cli/data_loader.rs` | core-engine::Value, `serde_json`, `serde_yaml` | データフォーマット変換（フォーマット依存はここに閉じる） |
| `cli/mod.rs`（CliRunner） | `cli/args.rs`, `cli/io.rs`, `cli/data_loader.rs`, core-engine::Mustache, core-engine::DirectoryPartialResolver | オーケストレーション |
| `main.rs` | `cli/mod.rs` | エントリポイント（薄い呼び出しのみ） |

## 外部クレート依存（ユニット別）

| 外部クレート | 利用ユニット | 用途 |
|---|---|---|
| `serde` | core-engine | `Value`の`Serialize`境界 |
| `serde_json` | cli | JSONデータのパース（`DataLoader`） |
| `serde_yaml` | cli | YAMLデータのパース（`DataLoader`） |
| `clap`（想定） | cli | コマンドライン引数解析（`CliArgs`） |
| `proptest`（想定、NFR-3） | core-engine（`tests/`配下） | プロパティベーステスト。フレームワーク最終確定はNFR Requirementsステージ |

外部クレートのフォーマット依存（`serde_json`/`serde_yaml`）がcliユニットに閉じており、core-engineユニットに波及しないことを確認（`application-design.md`のフォーマット非依存原則と一致）。

## 循環依存の検証
- core-engine ⇄ cli の循環依存なし（単方向のみ）
- クレート内モジュール間にも循環依存なし（`lib.rs`が全モジュールを束ねるファサードであり、各モジュールは相互に最小限の依存のみ）
