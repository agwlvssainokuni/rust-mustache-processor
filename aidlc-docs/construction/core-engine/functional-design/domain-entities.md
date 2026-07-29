# Domain Entities — core-engine

`components.md`・`component-methods.md`（Application Design）とFunctional Design Q1（数値表現）・Q5（エラー詳細度）の決定を踏まえた、core-engineの詳細データモデル。

## Value（データモデル）

フォーマット非依存の内部データ表現。JSON/YAMLいずれのデータもここに変換されてから core-engine に渡される（変換はcliの`DataLoader`が担当）。

> **注**: 本節は初版時点の記述。実装済みの正確な定義（`Map`型によるキー順序保持等）は`src/value.rs`を参照。以下は初版設計意図の記録として保持する。

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

### Value::Lambda（Mustacheオプションモジュール フルサポート、v0.2.0で追加）

`~lambdas`対応のため、`src/value.rs`の`Value`列挙型（現在`#[derive(Debug, Clone, PartialEq)]`）に新バリアントを追加する（`core-engine-mustache-optional-modules-functional-design-plan.md` Q1/Q4/Q5）。

```rust
pub enum Value {
    // ...既存バリアント（Null, Bool, Integer, Float, String, Array, Map）...
    Lambda(Rc<dyn Fn(&str) -> String>),
}
```

- 関数シグネチャは`Fn(&str) -> String`に統一する（BR-9.2）。セクション文脈では本体の生テキスト、変数展開文脈では空文字列`""`が渡される
- `Rc`を採用し`Send`/`Sync`境界は要求しない（BR-9.7）
- `Debug`/`Clone`/`PartialEq`は`Value`全体に対する手動実装が必要になる（クロージャは自動導出不可）。`Debug`は固定文字列（例: `"<lambda>"`）、`Clone`は`Rc`の参照カウント複製、`PartialEq`は`Lambda`が関わる比較を常に`false`とする（BR-9.6）
- ライブラリ利用者は`Value::Lambda(Rc::new(|text: &str| -> String { ... }))`のように構築する。CLIからは構築手段がない（データ形式に関数を表現できないため、FR-2）

## Node（内部AST、非公開）

Parserが生成し、Rendererが消費する中間表現。`lib.rs`からは非公開（`pub(crate)`）。

> **注**: 実装済みの正確な定義（`SourcePosition`のCopy実装等）は`src/ast.rs`を参照。以下はMustacheオプションモジュール フルサポート（v0.2.0）による拡張を含む設計意図の記録。

```rust
pub(crate) struct SourcePosition {
    pub line: usize,
    pub column: usize,
}

pub(crate) enum PartialName {
    Static(String),
    Dynamic(String), // Q9=A: {{>* name}} のみ対応。変数名を保持し、レンダリング時に文字列へ解決する
}

pub(crate) enum Node {
    Text(String),
    Variable { name: String, escape: bool, pos: SourcePosition },
    Section {
        name: String,
        inverted: bool,
        children: Vec<Node>,
        raw: String,          // Q6=A（v0.2.0追加）: セクション本体の生テキスト。ラムダのセクション文脈呼び出し（BR-9.2）に使用
        open: String,          // v0.2.0追加（fixture精査で発見）: このセクションタグが書かれた時点で有効だったデリミタ（開始側）
        close: String,         // 同上（終了側）。ラムダのセクション文脈での再パース（BR-9.3）に、デフォルトではなくこのデリミタを使う
        pos: SourcePosition,
    },
    Partial { name: PartialName, indent: String, pos: SourcePosition },
    // 以下2種はv0.2.0でテンプレート継承（~inheritance）対応のため追加
    Parent { name: String, children: Vec<Node>, indent: String, pos: SourcePosition },  // {{<parent}}...{{/parent}}。nameは常に静的（Q9=A）、indentはBR-10.6
    Block { name: String, children: Vec<Node>, pos: SourcePosition },  // {{$block}}...{{/block}}
}
```

- コメント（`{{! ... }}`）・デリミタ変更（`{{=<% %>=}}`）はASTノードとして保持しない。パース時に読み飛ばし、周辺のスタンドアロン行トリミング処理にのみ影響を与える
- `Partial`の`indent`は、パーシャルタグの直前にあった行頭空白（スタンドアロン判定された場合）を保持し、レンダリング時にパーシャル内容の各行へ適用する（`component-dependency.md`のパーシャルインデント処理）
- `pos`はQ5（エラーに行番号・列番号を含める）に基づき、`RenderError`生成時の位置情報として利用する
- `Section.raw`（v0.2.0追加）: パース時にセクション開始タグ直後〜終了タグ直前の元の文字列をそのまま保持する（Q6=A、再構築方式は採らない）。ラムダを参照しないテンプレートでは未使用だが、常に保持する（呼び出し時点でどのセクションがラムダを参照するかは静的に判別できないため）
- `Section.open`/`Section.close`（v0.2.0追加、fixture精査で発見）: パース時点でそのセクションタグに対して有効だったデリミタ文字列を保持する。ラムダのセクション文脈での返り値再パース（BR-9.3）にのみ使用する。インターポレーション文脈のラムダは常にデフォルトデリミタを使うため、`Variable`ノードには同種のフィールドは不要
- `Node::Parent`の本体（`children`）は、直下の`Node::Block`のみがオーバーライドとして意味を持ち、それ以外の内容（`Node::Text`等）は無視される（BR-10.2）
- `Node::Parent.indent`は`Node::Partial.indent`と同様、スタンドアロン時の行頭空白を保持し、親テンプレート文字列（値展開前）の各行に適用する（BR-10.6）
- `Node::Block`は`{{<parent}}`の内側（オーバーライド定義）と外側（単独評価、デフォルト内容の表示）の両方で同じ構造を使う（BR-10.4）

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
