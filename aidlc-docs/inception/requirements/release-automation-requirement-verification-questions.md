# Release Automation - Clarifying Questions

新規ユニット「release-automation」（GitHub Actionsによるクロスコンパイル・自動リリース）に関する確認質問です。各質問の[Answer]:タグに回答を記入してください。

## Question 1
リリースワークフローの起動トリガーはどれにしますか？

A) `v*.*.*`形式のGitタグをpushしたときに自動起動する

B) GitHub上で手動起動（workflow_dispatch）のみ

C) タグpush起動と手動起動の両方に対応する

D) Other (please describe after [Answer]: tag below)

[Answer]: C

**理由**: タグpush起動は「バージョンをタグ付けしたら自動でリリースされる」という一般的なRustプロジェクトの慣習に合致する。手動起動も併用できると、リリース失敗時の再実行や、正式タグを打つ前のビルド動作確認に使えて実用上便利。片方だけに絞る積極的な理由がない。

## Question 2
クロスコンパイル対象のプラットフォームはどれにしますか？

A) Linux x86_64のみ（最小構成）

B) Linux x86_64 + macOS（x86_64・aarch64） + Windows x86_64（主要3OS、Rust公式Tier1中心）

C) 上記Bに加えてLinux aarch64も含む（ARM Linuxサーバー・Raspberry Pi等も想定）

D) Other (please describe after [Answer]: tag below)

[Answer]: B

**理由**: README.mdで「クロスプラットフォーム配布」を謳っている以上、主要3OSはカバーすべき。Rust公式のTier 1ターゲットに絞ることで、クロスコンパイル環境（特にmacOSのaarch64）が安定して手に入りやすく、CI構築の複雑さも抑えられる。Linux aarch64は需要が確認できてから追加する形でも遅くない。

## Question 3
リリースのバージョン番号はどこから取得しますか？

A) pushされたGitタグ名から取得する（`Cargo.toml`のversionとは独立管理）

B) `Cargo.toml`の`version`フィールドを正とし、タグ名と一致するかCIで検証する

C) Other (please describe after [Answer]: tag below)

[Answer]: B

**理由**: `Cargo.toml`の`version`が唯一の真実の情報源（single source of truth）になっていれば、`cargo install`で入るバージョンとGitHub Releaseのバージョンが常に一致し、食い違いによる混乱を防げる。タグとの不一致をCIで検知してリリースを止めることで、タグ更新忘れ等のミスも防げる。

## Question 4
リリースノート（変更履歴）の生成方法はどうしますか？

A) GitHubの自動生成機能（`generate_release_notes`、直前タグからのPRタイトル一覧）に任せる

B) `CHANGELOG.md`等のファイルを別途手動管理し、その内容をリリース本文に転記する

C) Other (please describe after [Answer]: tag below)

[Answer]: A

**理由**: 現時点で`CHANGELOG.md`のような変更履歴ファイルはこのプロジェクトに存在せず、新たに手動運用を始めるのはCI設定というこのタスクのスコープを超える。GitHubの自動生成機能はPRタイトルから十分実用的なリリースノートを作れるため、まずはこれで始め、不足を感じたら後から`CHANGELOG.md`運用に発展させる方が段階的で無理がない。

## Question 5
配布アーカイブの形式・命名規則はどうしますか？

A) `mustache-<version>-<target-triple>.tar.gz`（Windowsのみ`.zip`）という一般的な命名規則を採用する

B) Other (please describe after [Answer]: tag below)

[Answer]: A

**理由**: Rustエコシステムで広く使われる命名慣習（`ripgrep`や`cargo-binstall`など多くのツールが採用）に従うことで、ユーザーが迷わず、将来`cargo-binstall`のような自動インストールツールとの親和性も確保できる。独自規則を発明する理由がない。

## Question 6
リリースビルドの前提として、テスト実行（`cargo test`）もワークフローに含めますか？

A) 含める（テスト失敗時はリリースを作成しない、ビルド失敗を未然に防ぐゲートとする）

B) 含めない（Build and Testステージで既に検証済みのため、リリースワークフローはビルド・パッケージングに専念する）

C) Other (please describe after [Answer]: tag below)

[Answer]: A

**理由**: 「Build and Testステージで検証済み」なのはそのコミット時点の話であり、リリースタグを打つタイミングでは追加のコミットが乗っている可能性がある。クロスコンパイル特有の環境差異（例: Windows特有のパス処理バグ）もあり得るため、リリース直前にもう一度テストゲートを通すのは低コストで安全性を高める妥当な保険。

## Question 7
crates.ioへの公開（`cargo publish`）もこのワークフローの対象に含めますか？

A) 含めない（今回はGitHub Releasesでのバイナリ配布のみを対象とする。crates.io公開は別途検討）

B) 含める（タグpush時にcrates.ioへも自動publishする）

C) Other (please describe after [Answer]: tag below)

[Answer]: A

**理由**: ユーザーの要望は明確に「クロスコンパイル・自動リリース」（バイナリ配布）であり、crates.io公開は別の関心事（ライブラリとしての配布戦略、APIの安定性コミットメントなど）を伴う別判断。スコープを広げすぎず、必要になったタイミングで別途Requirements Analysisを行う方がよい。
