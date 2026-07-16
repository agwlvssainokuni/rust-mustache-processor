# Business Logic Model — cli

## CliRunnerオーケストレーション処理

1. `argv`を`clap`で`CliArgs`にパースする（失敗時は`CliArgsError::Clap`として直ちにエラー処理へ、BR-1.1〜1.8の制約は`clap`の引数定義とその後のバリデーションで担保する）
2. `CliArgs`のバリデーション: BR-1.2/1.3（テンプレート指定の排他性・必須性）、BR-1.5（テンプレート・データ双方標準入力の禁止）を検査し、違反時は対応する`CliArgsError`を返す
3. `IoController::read_templates(args)`で、指定順に各テンプレート（ファイルまたは`--template-stdin`経由の標準入力）を読み込み、`Vec<LoadedTemplate>`を得る（BR-2.1〜2.3）
4. `IoController::read_data(args)`でデータ文字列を読み込む（`--data`指定時はファイル、未指定時は標準入力）
5. `DataLoader::detect_format(args)`でデータ形式を判定し（BR-3.1〜3.3）、`DataLoader::load(raw, format)`でcore-engineの`Value`に変換する（BR-3.4）
6. 出力バッファ（`String`）を初期化する
7. `Vec<LoadedTemplate>`を先頭から順に処理する。各`LoadedTemplate`について:
   a. `IoController::resolve_partials_dir(args, &loaded.source)`でパーシャルディレクトリを解決する（BR-4.1〜4.3）
   b. 解決したディレクトリで`DirectoryPartialResolver`を構築し、`Mustache::new().with_partial_resolver(...).with_strict(args.strict)`でエンジンを構築する（BR-1.7）
   c. `mustache.render_str(&loaded.content, &data)`でパース・レンダリングする（BR-5.1、各テンプレートは独立してパースされる）
   d. 成功した場合、結果を出力バッファに追記する（BR-5.2、セパレータなし）。失敗した場合、直ちにループを中断し`CliError::Mustache`を返す（BR-5.3、出力バッファは破棄し書き出さない）
8. 全テンプレートの処理が成功した場合のみ、`IoController::write_output(args, &buffer)`で出力先へ書き出す（BR-6.1）
9. 成功時は`ExitCode::from(0)`を返す（BR-7.1）。手順2〜8のいずれかで`CliError`が生じた場合、`mustache: {error}`形式でstderrへ出力し（BR-7.2〜7.3）、`ExitCode::from(1)`を返す

## エラー伝播

- `CliArgsError`/`IoError`/`DataLoaderError`/`mustache_processor::error::Error`はいずれも`CliError`に変換（`From`実装）され、`CliRunner::run`内で`?`演算子により一箇所（手順9）のエラーハンドリングに集約する
- core-engine側のエラー（`ParseError`/`RenderError`）は、その`Display`実装（行番号・列番号を含む）をそのまま`CliError`のメッセージとして利用する（新たな整形ロジックを追加しない）

## Testable Properties（PBT-01）

`aidlc-docs/inception/requirements/requirements.md`のNFR-3（PBT拡張機能: フル適用）に基づき、cliユニットで識別したテスト可能なプロパティ。cliユニットはファイルI/O・コマンドライン引数解析が中心であり、core-engineのパーサー/レンダラーのような純粋なアルゴリズム的性質は少ない。

| コンポーネント | プロパティ | カテゴリ | 内容 |
|---|---|---|---|
| DataLoader | JSON往復変換 | Round-trip | 任意の`serde_json::Value`互換データ（オブジェクト/配列/文字列/数値/真偽値/null）をJSON文字列化し、`DataLoader::load(_, DataFormat::Json)`でcore-engineの`Value`に変換した結果が、同じデータを直接`Value::from_serialize`で変換した結果と一致する |
| DataLoader | YAML往復変換 | Round-trip | 同上のYAML版。任意のデータをYAML文字列化し`DataLoader::load(_, DataFormat::Yaml)`で変換した結果が、直接変換した結果と一致する |
| DataLoader | 形式判定の決定性 | Idempotence | 同一の`CliArgs`に対して`detect_format`を複数回呼び出しても同じ結果を返す（副作用なし） |
| CliArgs | — | N/A | `clap`によるargv→構造体マッピングであり、独自のアルゴリズム的性質を持たない。妥当性はexample-basedテスト（BR-1.1〜1.8の代表ケース）で十分 |
| IoController | — | N/A | ファイルシステム・標準入出力への副作用が本質であり、テスト可能な普遍的性質を持たない。個別のI/Oシナリオはexample-basedテストで検証する |
| CliRunner | — | N/A | 上記コンポーネントのオーケストレーションのみで、独自の業務ロジックを持たない |

これらのプロパティはCode Generation（Planning）ステージでPBTテスト実装計画に引き継ぐ。
