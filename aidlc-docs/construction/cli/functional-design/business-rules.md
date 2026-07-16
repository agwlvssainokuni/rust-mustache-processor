# Business Rules — cli

`requirements.md`（FR-3, FR-5〜FR-8）と`cli-functional-design-plan.md`のQ1〜Q5・追加補正（`--data`フラグ）を統合した業務ルールカタログ。各ルールは`domain-entities.md`のエンティティを参照する。

## 1. CLI引数解析ルール

| ルール | 内容 |
|---|---|
| BR-1.1 | テンプレートは1つ以上の位置引数として指定できる（`Vec<PathBuf>`、`cat`ライクな複数指定、Question 1） |
| BR-1.2 | `--template-stdin`指定時、テンプレートの位置引数は省略する。位置引数と同時指定された場合は`CliArgsError::TemplateAndStdinConflict` |
| BR-1.3 | 位置引数・`--template-stdin`のいずれも指定されない場合は`CliArgsError::NoTemplateSpecified` |
| BR-1.4 | `--data <file>`未指定時、データは標準入力から読み込む（デフォルト、FR-5） |
| BR-1.5 | `--template-stdin`指定かつ`--data`未指定の場合、テンプレート・データ双方が標準入力を要求するため`CliArgsError::TemplateStdinAndDataStdinConflict`（FR-5） |
| BR-1.6 | `--output <file>`未指定時、出力は標準出力へ書き出す |
| BR-1.7 | `--strict`指定時、core-engineの`Mustache::with_strict(true)`を使用する（FR-7） |
| BR-1.8 | `--format json\|yaml`は明示指定可能（Question 2） |

## 2. テンプレート読み込みルール

| ルール | 内容 |
|---|---|
| BR-2.1 | テンプレート位置引数の各ファイルを、指定順にファイルとして読み込む |
| BR-2.2 | `--template-stdin`指定時、標準入力を単一テンプレートとして読み込む |
| BR-2.3 | いずれかのテンプレート読み込みに失敗した場合（ファイル不存在等）、`IoError`として即座に処理を中断する |

## 3. データ読み込み・形式判定ルール

| ルール | 内容 |
|---|---|
| BR-3.1 | データ形式判定は、`--format`明示指定を最優先とする |
| BR-3.2 | `--format`未指定時、`--data`で指定されたファイルの拡張子（`.json`→JSON, `.yaml`/`.yml`→YAML）で判定する |
| BR-3.3 | `--format`未指定かつ拡張子からの判定もできない場合（標準入力からのデータ読み込み等）、`DataLoaderError::UnknownFormat`（Question 2） |
| BR-3.4 | 判定したフォーマットで`serde_json`/`serde_yaml`によりパースし、`Value::from_serialize`でcore-engineの`Value`型に変換する |
| BR-3.5 | データは全テンプレートに対して共通の1つの値を使用する（テンプレートごとに個別のデータではない） |

## 4. パーシャルディレクトリ解決ルール

| ルール | 内容 |
|---|---|
| BR-4.1 | `--partials-dir`が明示指定されている場合、その値を全テンプレート共通で使用する |
| BR-4.2 | `--partials-dir`未指定時、位置引数で指定されたテンプレートファイルについては、そのファイル自身の親ディレクトリをデフォルトとする（テンプレートファイルごとに個別に解決する、Question 1派生点B） |
| BR-4.3 | `--partials-dir`未指定かつテンプレートが`--template-stdin`経由の場合、実行時のカレントディレクトリをデフォルトとする（Question 3） |

## 5. レンダリング・連結ルール

| ルール | 内容 |
|---|---|
| BR-5.1 | 各テンプレートを指定順に個別にパース・レンダリングする（各テンプレートは独立してパースされ、デリミタ変更等の状態は他のテンプレートに引き継がれない。Question 1で確定したprocess-then-cat方式） |
| BR-5.2 | 各テンプレートのレンダリング結果を、セパレータを挿入せず指定順に連結する（`cat`と同様の単純結合、Question 1派生点A） |
| BR-5.3 | いずれか1つのテンプレートでパース・レンダリングエラーが発生した場合、処理全体を直ちに中断し、それまでに得られた結果を含め一切出力しない（全体アトミック。core-engineの「レンダリング全体を中断、部分的な出力は返さない」方針との整合） |

## 6. 出力ルール

| ルール | 内容 |
|---|---|
| BR-6.1 | 全テンプレートのレンダリングが成功した場合のみ、連結結果を出力先（`--output`指定時はファイル、未指定時は標準出力）へ書き出す |

## 7. エラー処理・終了コードルール

| ルール | 内容 |
|---|---|
| BR-7.1 | 成功時の終了コードは0、エラー時は種別によらず1とする（Question 4） |
| BR-7.2 | エラー発生時、`mustache: <message>`形式の人間可読なプレーンテキストをstderrへ出力する（Question 5） |
| BR-7.3 | `<message>`は`CliError`の種別（CLI引数エラー・IOエラー・データ変換エラー・core-engineエラー）によらず、1行程度の人間可読なメッセージとする。core-engineの`ParseError`/`RenderError`は既存の`Display`実装をそのまま利用する |

## 8. 著作権・ライセンス表記ルール

| ルール | 内容 |
|---|---|
| BR-8.1 | cliユニットの全ソースファイルの先頭に、Apache License 2.0の著作権ヘッダー（著作権者: `agwlvssainokuni`）を付与する（FR-8, NFR-6。core-engineユニットと同一のヘッダー書式） |
