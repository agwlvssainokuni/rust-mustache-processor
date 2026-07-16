# Component Methods — rust-mustache-processor

概要レベルのメソッドシグネチャ。詳細な業務ルール（真偽判定の厳密な条件分岐、エスケープ規則の細部等）はFunctional Design（per-unit, CONSTRUCTION phase）で定義する。

## ユニット: core-engine

### Value
| メソッド | シグネチャ（概要） | 目的 |
|---|---|---|
| `from_serialize` | `fn from_serialize<T: Serialize>(value: &T) -> Result<Value, ValueError>` | 任意のSerialize実装型からValueへ変換 |
| `is_truthy` | `fn is_truthy(&self) -> bool` | セクション評価用の真偽判定（Mustache仕様のfalsy定義に従う） |
| `get` | `fn get(&self, key: &str) -> Option<&Value>` | Mapからのキー参照（コンテキスト探索に使用） |
| `iter` | `fn iter(&self) -> Option<impl Iterator<Item = &Value>>` | Arrayの場合の繰り返し評価用イテレータ |

### Mustache（エンジン）
| メソッド | シグネチャ（概要） | 目的 |
|---|---|---|
| `new` | `fn new() -> Self` | デフォルト設定（リゾルバなし、strict=false）でエンジンを作成 |
| `with_partial_resolver` | `fn with_partial_resolver(self, resolver: Box<dyn PartialResolver>) -> Self` | パーシャルリゾルバを設定（ビルダースタイル） |
| `with_strict` | `fn with_strict(self, strict: bool) -> Self` | 未定義変数をエラーにするstrictモードを設定（FR-7） |
| `parse` | `fn parse(&self, template: &str) -> Result<Template, ParseError>` | テンプレート文字列をパースし再利用可能な`Template`を得る |
| `render` | `fn render(&self, template: &Template, data: &Value) -> Result<String, RenderError>` | パース済みテンプレートをデータでレンダリング |
| `render_str` | `fn render_str(&self, template: &str, data: &Value) -> Result<String, Error>` | パース＋レンダリングを1回で行う簡易メソッド（`Error`は`ParseError`/`RenderError`を包む統合型） |

### Parser
| メソッド | シグネチャ（概要） | 目的 |
|---|---|---|
| `parse` | `fn parse(template: &str) -> Result<Vec<Node>, ParseError>` | テンプレート文字列からASTノード列を生成 |

### Renderer
| メソッド | シグネチャ（概要） | 目的 |
|---|---|---|
| `render` | `fn render(nodes: &[Node], data: &Value, resolver: Option<&dyn PartialResolver>, strict: bool) -> Result<String, RenderError>` | ASTをコンテキストに対して評価し出力文字列を生成 |

### PartialResolver（トレイト）
| メソッド | シグネチャ（概要） | 目的 |
|---|---|---|
| `resolve` | `fn resolve(&self, name: &str) -> Option<String>` | パーシャル名からテンプレート文字列を取得（未解決時は`None`） |

### DirectoryPartialResolver
| メソッド | シグネチャ（概要） | 目的 |
|---|---|---|
| `new` | `fn new(dir: PathBuf) -> Self` | パーシャル探索対象ディレクトリを指定して生成 |
| `resolve`（トレイト実装） | `fn resolve(&self, name: &str) -> Option<String>` | ディレクトリ配下のファイルを読み込んで返す |

## ユニット: cli

### CliArgs
| メソッド | シグネチャ（概要） | 目的 |
|---|---|---|
| `parse_args` | `fn parse_args(argv: &[String]) -> Result<CliArgs, CliArgsError>` | コマンドライン引数を解析 |

### IoController
| メソッド | シグネチャ（概要） | 目的 |
|---|---|---|
| `read_template` | `fn read_template(args: &CliArgs) -> Result<String, IoError>` | テンプレートをファイルまたは標準入力から読み込む |
| `read_data` | `fn read_data(args: &CliArgs) -> Result<String, IoError>` | データをファイルまたは標準入力から読み込む（デフォルト標準入力） |
| `resolve_partials_dir` | `fn resolve_partials_dir(args: &CliArgs) -> PathBuf` | パーシャルディレクトリを解決（未指定時はテンプレートファイルのディレクトリ、FR-6） |
| `write_output` | `fn write_output(args: &CliArgs, content: &str) -> Result<(), IoError>` | レンダリング結果をファイルまたは標準出力へ書き出す |

### DataLoader
| メソッド | シグネチャ（概要） | 目的 |
|---|---|---|
| `load` | `fn load(raw: &str, format: DataFormat) -> Result<Value, DataLoaderError>` | JSON/YAML文字列をパースしcore-engineの`Value`へ変換 |
| `detect_format` | `fn detect_format(args: &CliArgs) -> Result<DataFormat, DataLoaderError>` | 拡張子または明示オプションから形式を判定 |

### CliRunner
| メソッド | シグネチャ（概要） | 目的 |
|---|---|---|
| `run` | `fn run(argv: &[String]) -> ExitCode` | CLIのメインフローを実行し、終了コードを返す |