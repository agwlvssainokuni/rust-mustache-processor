# Release Automation - Clarifying Questions

新規ユニット「release-automation」（GitHub Actionsによるクロスコンパイル・自動リリース）に関する確認質問です。各質問の[Answer]:タグに回答を記入してください。

## Question 1
リリースワークフローの起動トリガーはどれにしますか？

A) `v*.*.*`形式のGitタグをpushしたときに自動起動する

B) GitHub上で手動起動（workflow_dispatch）のみ

C) タグpush起動と手動起動の両方に対応する

D) Other (please describe after [Answer]: tag below)

[Answer]: 

## Question 2
クロスコンパイル対象のプラットフォームはどれにしますか？

A) Linux x86_64のみ（最小構成）

B) Linux x86_64 + macOS（x86_64・aarch64） + Windows x86_64（主要3OS、Rust公式Tier1中心）

C) 上記Bに加えてLinux aarch64も含む（ARM Linuxサーバー・Raspberry Pi等も想定）

D) Other (please describe after [Answer]: tag below)

[Answer]: 

## Question 3
リリースのバージョン番号はどこから取得しますか？

A) pushされたGitタグ名から取得する（`Cargo.toml`のversionとは独立管理）

B) `Cargo.toml`の`version`フィールドを正とし、タグ名と一致するかCIで検証する

C) Other (please describe after [Answer]: tag below)

[Answer]: 

## Question 4
リリースノート（変更履歴）の生成方法はどうしますか？

A) GitHubの自動生成機能（`generate_release_notes`、直前タグからのPRタイトル一覧）に任せる

B) `CHANGELOG.md`等のファイルを別途手動管理し、その内容をリリース本文に転記する

C) Other (please describe after [Answer]: tag below)

[Answer]: 

## Question 5
配布アーカイブの形式・命名規則はどうしますか？

A) `mustache-<version>-<target-triple>.tar.gz`（Windowsのみ`.zip`）という一般的な命名規則を採用する

B) Other (please describe after [Answer]: tag below)

[Answer]: 

## Question 6
リリースビルドの前提として、テスト実行（`cargo test`）もワークフローに含めますか？

A) 含める（テスト失敗時はリリースを作成しない、ビルド失敗を未然に防ぐゲートとする）

B) 含めない（Build and Testステージで既に検証済みのため、リリースワークフローはビルド・パッケージングに専念する）

C) Other (please describe after [Answer]: tag below)

[Answer]: 

## Question 7
crates.ioへの公開（`cargo publish`）もこのワークフローの対象に含めますか？

A) 含めない（今回はGitHub Releasesでのバイナリ配布のみを対象とする。crates.io公開は別途検討）

B) 含める（タグpush時にcrates.ioへも自動publishする）

C) Other (please describe after [Answer]: tag below)

[Answer]: 
