# Domain Entities — core-engine

`components.md`・`component-methods.md`（Application Design）とFunctional Design Q1（数値表現）・Q5（エラー詳細度）の決定を踏まえた、core-engineの詳細データモデル。

## Value（データモデル）

フォーマット非依存の内部データ表現。JSON/YAMLいずれのデータもここに変換されてから core-engine に渡される（変換はcliの`DataLoader`が担当）。

```rust
pub enum Value {
    Null,
    Bool(bool),
    Integer(i64),   // Q1=B: 整数と浮動小数点数を区別
    Float(f64),
    String(String),
    Array(Vec<Value>),
    Object(HashMap<String, Value>),
}
```

- `Object`のキー順序は仕様上の意味を持たない（セクションはハッシュを1回のコンテキストとして扱うのみで、キーを列挙する操作はないため）。よって順序非保証の`HashMap`で十分
- `Integer`/`Float`の区別はレンダリング時の文字列化（`business-rules.md`参照）にのみ影響し、真偽判定（`business-rules.md`のセクション真偽ルール）には影響しない（`0`も`0.0`も、Mustache仕様上は非0の数値なので真として扱われる）

## Node（内部AST、非公開）

Parserが生成し、Rendererが消費する中間表現。`lib.rs`からは非公開（`pub(crate)`）。

```rust
pub(crate) struct SourcePosition {
    pub line: usize,
    pub column: usize,
}

pub(crate) enum Node {
    Text(String),
    Variable { name: String, escape: bool, pos: SourcePosition },
    Section { name: String, inverted: bool, children: Vec<Node>, pos: SourcePosition },
    Partial { name: String, indent: String, pos: SourcePosition },
}
```

- コメント（`{{! ... }}`）・デリミタ変更（`{{=<% %>=}}`）はASTノードとして保持しない。パース時に読み飛ばし、周辺のスタンドアロン行トリミング処理にのみ影響を与える
- `Partial`の`indent`は、パーシャルタグの直前にあった行頭空白（スタンドアロン判定された場合）を保持し、レンダリング時にパーシャル内容の各行へ適用する（`component-dependency.md`のパーシャルインデント処理）
- `pos`はQ5（エラーに行番号・列番号を含める）に基づき、`RenderError`生成時の位置情報として利用する

## Template（公開）

```rust
pub struct Template {
    pub(crate) root: Vec<Node>,
}
```

パース結果を保持する不透明な値。内部構造（`Node`）は公開せず、`Mustache::render`にのみ渡せる。

> **Step8での補正**: 以下の`PartialResolver`（`Result`ベース）と`RenderErrorKind`（`PartialCycleDetected`を含む3種）は初版の設計であり、その後Application Design（`component-methods.md`）準拠への補正（Step5、`Option<String>`ベースに変更）およびStep8のspec conformanceテストで判明した補正（`PartialParseError`/`MaxNestingDepthExceeded`の追加、`PartialCycleDetected`の削除）により変更されている。現在の正確な定義は`aidlc-docs/construction/core-engine/nfr-design/logical-components.md`と実装（`src/error.rs`, `src/partial.rs`）を参照。

## PartialResolver（公開トレイト）

```rust
pub trait PartialResolver {
    fn resolve(&self, name: &str) -> Result<String, PartialResolveError>;
}

pub struct PartialResolveError {
    pub name: String,
    pub message: String,
}
```

- `resolve`が`Err`を返した場合、Rendererはそれを`RenderError`（Q3=A: strictモードに関わらず常にエラー）に変換する

## DirectoryPartialResolver（公開実装）

```rust
pub struct DirectoryPartialResolver {
    base_dir: PathBuf,
}

impl DirectoryPartialResolver {
    pub fn new(base_dir: impl Into<PathBuf>) -> Self;
}

impl PartialResolver for DirectoryPartialResolver {
    fn resolve(&self, name: &str) -> Result<String, PartialResolveError> {
        // base_dir.join(format!("{name}.mustache")) を読み込む
    }
}
```

## エラー型（公開）

Q5（行番号・列番号を含める）に基づく設計:

```rust
pub struct ParseError {
    pub kind: ParseErrorKind,
    pub line: usize,
    pub column: usize,
    pub message: String,
}

pub enum ParseErrorKind {
    UnexpectedEof,
    UnbalancedSection { name: String },
    UnknownDelimiterSyntax,
    // ...
}

pub struct RenderError {
    pub kind: RenderErrorKind,
    pub line: usize,
    pub column: usize,
    pub message: String,
}

pub enum RenderErrorKind {
    UndefinedVariable { name: String },     // strictモード時のみ発生（Q2=A: 変数展開のみ対象）
    PartialNotFound { name: String },        // 常に発生（Q3=A）
    PartialCycleDetected { chain: Vec<String> }, // Q4=B
}

pub enum Error {
    Parse(ParseError),
    Render(RenderError),
}
```

## Mustache（公開エンジン）

```rust
pub struct Mustache {
    partial_resolver: Option<Box<dyn PartialResolver>>,
    strict: bool,
}
```

`component-methods.md`のシグネチャをそのまま踏襲（変更なし）。

## エンティティ関連図（テキスト表現）

```
Mustache ──uses──> Parser (internal) ──produces──> Template (root: Vec<Node>)
Mustache ──uses──> Renderer (internal)
Renderer ──reads──> Template, Value
Renderer ──calls──> PartialResolver (trait)
Renderer ──on partial──> Parser (再帰的に再パース)
DirectoryPartialResolver ──implements──> PartialResolver
Node::Variable / Node::Section / Node::Partial ──each has──> SourcePosition
ParseError / RenderError ──carry──> line, column (from SourcePosition)
```
