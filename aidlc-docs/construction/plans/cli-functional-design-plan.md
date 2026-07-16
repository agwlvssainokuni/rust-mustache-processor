# Functional Design Plan — cli

`requirements.md`（FR-5〜FR-7）、`component-methods.md`、`unit-of-work-dependency.md`を踏まえた、cliユニットの詳細設計に向けた計画。

## 前提（要件で決定済み、質問不要）

- データのデフォルト入力元は標準入力、テンプレートは常にファイル引数（デフォルト経路）（FR-5）
- テンプレート・データ双方が同時に標準入力に指定された場合はエラー（FR-5）
- `--partials-dir`未指定時、テンプレートファイルが位置引数で指定されている場合はそのディレクトリをデフォルトとする（FR-6）
- 未定義変数はデフォルトで継続、strictモード相当のオプションでエラー化可能（FR-7、core-engineの`Mustache::with_strict`を呼び出すのみ）
- 出力先はファイル指定可能（FR-5）
- JSON/YAMLをサポート、TOML等は対象外（FR-3）
- 外部クレート: `clap`（引数解析）、`serde_json`/`serde_yaml`（データ変換）（`unit-of-work-dependency.md`）

## Plan Checklist

- [ ] Step 1: 要件・Application Design成果物の分析（本ファイル作成）
- [ ] Step 2-4: 未決定論点の洗い出し・質問提示（本ファイル）
- [ ] Step 5: ユーザー回答収集・曖昧さ分析
- [ ] Step 6: Functional Design成果物生成（domain-entities.md, business-rules.md, business-logic-model.md）
- [ ] Step 7-9: 完了メッセージ提示・承認待ち・記録

## 決定が必要な論点（質問）

### Question 1: CLI引数の具体的な形状

`component-methods.md`は`CliArgs::parse_args(argv) -> Result<CliArgs, CliArgsError>`という関数シグネチャのみを定義しており、具体的な引数の並び（位置引数かフラグのみか）は未決定。

A) 位置引数 + フラグ方式: `mustache <template-file> [data-file] [-o/--output <file>] [--partials-dir <dir>] [--strict] [--format json|yaml] [--template-stdin]`。テンプレートは第1位置引数（または`--template-stdin`指定時は省略）、データは第2位置引数（省略時は標準入力）。FR-5の例示コマンド`mustache template.tmpl < data.json`と自然に合致する

B) フラグのみ方式: `mustache --template <file> [--data <file>] [--output <file>] ...`。位置引数を一切使わず全て明示的なフラグで指定する。冗長だが曖昧さがない

C) Other（[Answer]: タグの下に詳細を記載）

[Answer]:

### Question 2: データ形式（JSON/YAML）判定の優先順位

`component-methods.md`の`DataLoader::detect_format(args) -> Result<DataFormat, DataLoaderError>`は判定に失敗しうることを示しているが、優先順位は未決定。

A) 明示`--format`オプションを最優先。未指定時はデータファイルの拡張子（`.json`/`.yaml`/`.yml`）で判定。データが標準入力等で拡張子が得られず`--format`も未指定の場合は`DataLoaderError`とする

B) 拡張子判定を優先し、`--format`は拡張子が無い・不明な場合のみのフォールバックとして使う

C) Other（[Answer]: タグの下に詳細を記載）

[Answer]:

### Question 3: テンプレートの標準入力切替とパーシャルディレクトリのデフォルト解決

FR-5は「オプション指定時、テンプレート側を標準入力から読み込む切り替えが可能」としているが、具体的な指定方法とその際のパーシャルディレクトリのデフォルト解決（FR-6の「テンプレートファイルのディレクトリ」が存在しないケース）が未決定。

A) `--template-stdin`フラグで明示的に切り替える（この場合テンプレートの位置引数は省略。両方指定された場合はエラー）。`--partials-dir`未指定時は実行時のカレントディレクトリへフォールバックする

B) テンプレート位置引数に`-`を渡すUnix慣習を採用する。`--partials-dir`未指定時は同様にカレントディレクトリへフォールバックする

C) Other（[Answer]: タグの下に詳細を記載）

[Answer]:

### Question 4: 終了コード方針

`component-methods.md`の`CliRunner::run(argv) -> ExitCode`はコード体系までは規定していない。

A) 成功時0、それ以外は全て1とするシンプルな二値方式。エラー種別の詳細はstderrメッセージで判別する（Unix慣習、YAGNI）

B) エラー種別ごとに異なる非ゼロコード（引数エラー=2、入出力エラー=3、データ変換エラー=4、パース/レンダリングエラー=5等）を割り当て、シェルスクリプトからの分岐判定を可能にする

C) Other（[Answer]: タグの下に詳細を記載）

[Answer]:

### Question 5: エラーメッセージの出力形式

A) `mustache: <message>`のようにプログラム名を前置した人間可読なプレーンテキストをstderrへ出力する（1〜数行）

B) JSON等の構造化形式でエラー詳細（種別・位置情報等）を出力する

C) Other（[Answer]: タグの下に詳細を記載）

[Answer]:
