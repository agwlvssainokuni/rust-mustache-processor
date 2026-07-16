# Domain Entities — cli

`components.md`・`component-methods.md`（Application Design）と`cli-functional-design-plan.md`のQ1〜Q5・追加補正（`--data`フラグ）の決定を踏まえた、cliユニットの詳細データモデル。

## CliArgs（引数解析結果）

```rust
pub(crate) struct CliArgs {
    pub(crate) templates: Vec<PathBuf>,       // 1つ以上（template_stdin=trueの場合は空）
    pub(crate) template_stdin: bool,
    pub(crate) data: Option<PathBuf>,         // Noneなら標準入力
    pub(crate) output: Option<PathBuf>,       // Noneなら標準出力
    pub(crate) partials_dir: Option<PathBuf>, // Some時は全テンプレート共通で使用
    pub(crate) strict: bool,
    pub(crate) format: Option<DataFormat>,
}

pub(crate) enum DataFormat {
    Json,
    Yaml,
}
```

- `templates`が複数指定された場合、指定順にファイルとして扱う（Question 1、`cat`ライクな複数テンプレート対応）
- `component-methods.md`の`parse_args(argv: &[String]) -> Result<CliArgs, CliArgsError>`のシグネチャをそのまま踏襲

## CliArgsError（公開不要、cliバイナリ内部エラー）

```rust
pub(crate) enum CliArgsError {
    Clap(clap::Error),                    // clap自体の解析失敗（--help/--version表示含む）
    NoTemplateSpecified,                  // 位置引数も--template-stdinもない
    TemplateAndStdinConflict,             // 位置引数と--template-stdin両方指定
    TemplateStdinAndDataStdinConflict,    // --template-stdinかつ--data未指定（両方標準入力）
    InvalidFormat { value: String },      // --formatの値が"json"/"yaml"以外
}
```

**実装時の補正**: `CliArgsError`に`#[derive(PartialEq)]`を付与してユニットテストで`assert_eq!`によるバリアント比較を可能にしたいが、`clap::Error`は`PartialEq`を実装していないため、`Clap(clap::Error)`のままでは`CliArgsError`全体を`PartialEq`にできない。実装では`Clap(String)`（`clap::Error`の`to_string()`結果）に変更した。`detect_format`も同様の理由で`&CliArgs`ではなく必要なフィールド（`explicit_format: Option<DataFormat>`, `data_path: Option<&Path>`）を直接引数に取る形に詳細化した（`CliArgs`が`data_loader.rs`の`DataFormat`型に依存するため、逆に`detect_format`が`&CliArgs`を取ると`args`⇄`data_loader`モジュール間の循環依存が生じるため）。

## IoController（入出力制御）

```rust
pub(crate) enum TemplateSource {
    File(PathBuf),
    Stdin,
}

pub(crate) struct LoadedTemplate {
    pub(crate) source: TemplateSource,
    pub(crate) content: String,
}

pub(crate) fn read_templates(args: &CliArgs) -> Result<Vec<LoadedTemplate>, IoError>;
pub(crate) fn read_data(args: &CliArgs) -> Result<String, IoError>;
pub(crate) fn resolve_partials_dir(args: &CliArgs, source: &TemplateSource) -> PathBuf;
pub(crate) fn write_output(args: &CliArgs, content: &str) -> Result<(), IoError>;
```

- `resolve_partials_dir`は`component-methods.md`の`fn resolve_partials_dir(args: &CliArgs) -> PathBuf`を、複数テンプレート・ファイルごとのデフォルト解決（Question 1派生点B）に対応させるため`source: &TemplateSource`引数を追加する形に詳細化した
- `LoadedTemplate`はconcept-levelの`component-methods.md`には存在しない、Functional Designで導入した詳細型（各テンプレートの内容とパーシャルディレクトリ解決に必要な出所情報を1組で保持する）

```rust
pub(crate) enum IoError {
    TemplateRead { path: PathBuf, message: String },
    TemplateStdinRead { message: String },
    DataRead { message: String },   // ファイル or 標準入力
    OutputWrite { message: String },
}
```

## DataLoader（データローダー）

```rust
pub(crate) fn detect_format(args: &CliArgs) -> Result<DataFormat, DataLoaderError>;
pub(crate) fn load(raw: &str, format: DataFormat) -> Result<mustache_processor::value::Value, DataLoaderError>;

pub(crate) enum DataLoaderError {
    UnknownFormat,                                    // --format未指定かつ拡張子から判別不能
    Parse { format: DataFormat, message: String },
}
```

- `component-methods.md`のシグネチャをそのまま踏襲。`load`はcore-engineの`Value::from_serialize`を内部で呼び出し、`serde_json::Value`/`serde_yaml::Value`から変換する

## CliRunner（オーケストレーション）

```rust
pub(crate) fn run(argv: &[String]) -> std::process::ExitCode;
```

- `component-methods.md`のシグネチャをそのまま踏襲。内部でCliArgs解析→IoController→DataLoader→core-engine::Mustache→IoControllerの順に実行する（詳細は`business-logic-model.md`参照）

## CliError（統合エラー型、cliバイナリ内部）

```rust
pub(crate) enum CliError {
    Args(CliArgsError),
    Io(IoError),
    DataLoader(DataLoaderError),
    Mustache(mustache_processor::error::Error), // core-engineのParseError/RenderErrorを統合するError型
}
```

- `component-methods.md`には存在しない、Functional Designで導入した内部エラー統合型。core-engineの`Error`型と同様のパターンを踏襲し、`CliRunner::run`内で`?`演算子によるエラー伝播を可能にする
- `std::fmt::Display`を実装し、BR-7.2（`mustache: <message>`形式）のメッセージ整形に用いる。`Mustache(e)`の場合は`e`（`mustache_processor::error::Error`）が既に`Display`を実装しているため、そのメッセージをそのまま利用する

## エンティティ関連図（テキスト表現）

```
main.rs ──calls──> CliRunner::run
CliRunner ──uses──> CliArgs::parse_args (clap)
CliRunner ──uses──> IoController::read_templates / read_data / resolve_partials_dir / write_output
CliRunner ──uses──> DataLoader::detect_format / load
CliRunner ──uses──> mustache_processor::Mustache, DirectoryPartialResolver（テンプレートごとに構築）
CliRunner ──on error──> CliError ──Display──> BR-7.2のメッセージ形式でstderrへ
```
