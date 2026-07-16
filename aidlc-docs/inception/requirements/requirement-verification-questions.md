# Requirements Clarification Questions — rust-mustache-processor

以下の質問に回答してください。各質問の`[Answer]:`タグの後に選択肢の記号（A, B, C...）を記入してください。選択肢に該当するものがない場合は最後の「Other」を選び、内容を記述してください。

## Question 0: Mustache処理エンジンの実装方針
Mustacheのパース・レンダリング処理はどう実装しますか？

A) 既存クレート（`mustache`, `ramhorns`等）をラップして利用する

B) 独自にMustache処理エンジン（パーサー・レンダラー）を自作する

X) Other (please describe after [Answer]: tag below)

[Answer]: B

**補足（チャットでの合意事項）**: プロジェクト名が示す通り、Mustache処理系の実装そのものが本プロジェクトの目的であるため自作する。既存クレートは`mustache`（開発停滞気味）、`ramhorns`（高速だが静的テンプレート前提で仕様から一部逸脱）があり、ラップするだけでは仕様準拠範囲やパーシャル解決等の要件検討が意味を持たなくなるため採用しない。自分でコントロールできることを重視する、との判断。

## Question 1: プロダクトの目的・提供形態
このツールは何として提供しますか？

A) CLIツール — テンプレートファイルとデータファイルを受け取り、レンダリング結果を出力するコマンド

B) ライブラリ — 他のRustプログラムから呼び出して使うクレート

C) CLIツール＋ライブラリ両方（コア機能をライブラリ化し、CLIはその薄いラッパー）

X) Other (please describe after [Answer]: tag below)

[Answer]: C

## Question 2: 入力データ形式
テンプレートに埋め込むデータはどの形式で受け取りますか？

A) JSONのみ

B) JSONとYAML

C) JSON・YAML・TOMLなど複数形式に対応

X) Other (please describe after [Answer]: tag below)

[Answer]: B

## Question 3: Mustache仕様への準拠範囲
どの程度Mustache仕様に準拠しますか？

A) 基本機能のみ（変数展開、セクション、コメント）

B) 標準機能一式（変数、セクション、逆セクション、パーシャル、コメント、デリミタ変更）

C) 完全準拠（公式mustache/specのコンフォーマンステストスイートに準拠、ラムダ含む）

X) Other (please describe after [Answer]: tag below)

[Answer]: B

## Question 4: CLIの入出力インターフェース
CLIとしての入出力方法はどうしますか？

A) ファイル指定のみ（テンプレートファイル・データファイル・出力先ファイルを引数指定）

B) ファイル指定に加え、標準入力・標準出力にも対応（パイプ処理可能）

C) さらに設定ファイル（一括変換のバッチ処理定義）にも対応

X) Other (please describe after [Answer]: tag below)

[Answer]: B

**補足（チャットでの合意事項）**: 標準入力のデフォルト対象はデータとする（`mustache template.tmpl < data.json`）。テンプレートを標準入力から読みたい場合は、コマンドラインオプションで明示的に切り替え可能とする（例: `--template-stdin` 相当のオプション）。テンプレート・データ双方に標準入力を同時に割り当てることはできない（単一ストリームのため）ので、その組み合わせはエラーとする。

## Question 5: パーシャル（部分テンプレート）の解決方法
パーシャル（`{{> partial}}`）を使う場合、テンプレートはどこから読み込みますか？

A) パーシャル機能は不要（対象外）

B) 指定ディレクトリ内のファイルから読み込み

C) 事前にすべてメモリ上にロードされたテンプレート集合から解決

X) Other (please describe after [Answer]: tag below)

[Answer]: B

**補足（チャットでの合意事項）**: パーシャル用ディレクトリは相対パス・絶対パスどちらも指定可能（相対パスはカレントディレクトリ基準で解決）。`--partials-dir`未指定時のデフォルトは、メインテンプレートファイルが置かれているディレクトリとする（実行時のカレントディレクトリではなく、テンプレート一式の相対的な位置関係を保てるようにするため）。

## Question 6: 未定義変数・エラー時の挙動
テンプレート中の変数がデータに存在しない場合、どう振る舞いますか？

A) 空文字として扱い処理を継続（Mustache仕様のデフォルト挙動）

B) エラーとして処理を中断する

C) 既定は継続だが、オプション（strictモード等）でエラーにも切り替え可能

X) Other (please describe after [Answer]: tag below)

[Answer]: C

## Question 7: 非機能要件（性能・配布形態）
性能や配布に関して特に考慮すべき点はありますか？

A) 特になし（一般的なCLIツールとしての妥当な性能で良い）

B) 大容量テンプレート・データ（数十MB以上）を高速に処理できる必要がある

C) シングルバイナリとしてクロスプラットフォーム配布（cargo install / GitHub Releases等）を想定している

X) Other (please describe after [Answer]: tag below)

[Answer]: C

## Question 8: テスト方針
品質保証としてどこまで求めますか？

A) 標準的な単体テスト・結合テストで十分

B) 上記に加え、公式mustache/specのコンフォーマンステストスイート（JSON形式）を取り込んで準拠検証する

X) Other (please describe after [Answer]: tag below)

[Answer]: B

**補足（チャットでの合意事項）**: Q3=Bでラムダ（`~lambdas`）を対象外としたため、公式spec取り込みも必須モジュール（comments, delimiters, interpolation, inverted, partials, sections）を対象とし、オプションモジュール（ラムダ等）のテストケースは対象外とする。

---

## 拡張機能のオプトイン確認

以下は本ワークフローに用意されている追加ルールセット（拡張機能）の適用可否です。

## Question: Security Extensions
Should security extension rules be enforced for this project?

A) Yes — enforce all SECURITY rules as blocking constraints (recommended for production-grade applications)

B) No — skip all SECURITY rules (suitable for PoCs, prototypes, and experimental projects)

X) Other (please describe after [Answer]: tag below)

[Answer]: B

## Question: Resiliency Extensions
Should the resiliency baseline be applied to this project?

**What this extension is.** Enabling it applies a set of directional, design-time best practices for building resilient systems, derived from the AWS Well-Architected Framework (Reliability Pillar). It steers requirements, design, and code toward fault tolerance, high availability, observability, and recoverability.

**What this extension is NOT.** Enabling it does not make your workload production-ready, nor certify any availability/RTO/RPO target.

A) Yes — apply the resiliency baseline as directional best practices and design-time guidance

B) No — skip the resiliency baseline (suitable for PoCs, prototypes, and experimental projects)

X) Other (please describe after [Answer]: tag below)

[Answer]: B

## Question: Property-Based Testing Extension
Should property-based testing (PBT) rules be enforced for this project?

A) Yes — enforce all PBT rules as blocking constraints (recommended for projects with business logic, data transformations, serialization, or stateful components)

B) Partial — enforce PBT rules only for pure functions and serialization round-trips (suitable for projects with limited algorithmic complexity)

C) No — skip all PBT rules (suitable for simple CRUD applications, UI-only projects, or thin integration layers with no significant business logic)

X) Other (please describe after [Answer]: tag below)

[Answer]: A
