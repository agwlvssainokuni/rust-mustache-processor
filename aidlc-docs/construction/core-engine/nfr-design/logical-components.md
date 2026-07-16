# Logical Components — core-engine

`nfr-design-patterns.md`のパターンを実現する論理コンポーネント（非公開の内部実装単位）。`domain-entities.md`（Functional Design）を拡張する。

## RenderState（新規、非公開）

Rendererの再帰呼び出し全体で共有・更新される内部状態。`nfr-design-patterns.md`パターン1〜3に対応。

```rust
pub(crate) struct RenderState<'a> {
    context_stack: Vec<&'a Value>,
    depth: usize,
    partial_chain: Vec<String>,
    strict: bool,
}

pub(crate) const MAX_NESTING_DEPTH: usize = 1000;
```

- `context_stack`: Functional Designで定義済みのコンテキストスタック（BR-4.1/BR-4.2）を`RenderState`に格納する形に整理
- `depth`: セクション・パーシャルいずれの再帰前後でインクリメント/デクリメント。`MAX_NESTING_DEPTH`超過時にエラーとする（パターン1）
- `partial_chain`: 解決中のパーシャル名を保持し、循環検出に使う（パターン2）
- `strict`: `Mustache`エンジンの設定値をレンダリング全体で参照するためのコピー

## RenderErrorKind の拡張

`domain-entities.md`（Functional Design）で定義済みの`RenderErrorKind`に、ネスト深度超過用のバリアントを追加する:

```rust
pub enum RenderErrorKind {
    UndefinedVariable { name: String },
    PartialNotFound { name: String },
    PartialCycleDetected { chain: Vec<String> },
    MaxNestingDepthExceeded { depth: usize }, // 追加
}
```

## Renderer 内部関数シグネチャの更新

`business-logic-model.md`のレンダリング処理を、`RenderState`を用いる形に具体化する:

```rust
pub(crate) fn render_nodes(
    nodes: &[Node],
    state: &mut RenderState,
    partial_resolver: Option<&dyn PartialResolver>,
    out: &mut String,
) -> Result<(), RenderError>;
```

- 単一の`&mut RenderState`を引き回すことで、Question 1（NFR Design）の決定を反映
- `out: &mut String`は、パターン4（事前確保済みバッファ）をエントリポイント（`Mustache::render`）から渡し続けるための引数

## Mustache::render エントリポイントの更新

```rust
impl Mustache {
    pub fn render(&self, template: &Template, data: &Value) -> Result<String, RenderError> {
        let mut out = String::with_capacity(template.source_len); // パターン4
        let mut state = RenderState::new(data, self.strict);       // depth=0, partial_chain=空で初期化
        render_nodes(&template.root, &mut state, self.partial_resolver.as_deref(), &mut out)?;
        Ok(out)
    }
}
```

- `Template`に`source_len: usize`（パース元テンプレートのバイト長）を追加保持する（`domain-entities.md`の`Template`定義を拡張）

## テスト用論理コンポーネント（PBT関連）

| コンポーネント | 配置 | 用途 |
|---|---|---|
| `tests/spec/` | `tests/`配下 | 公式mustache/specコンフォーマンステストのJSONケース取り込み・実行 |
| `tests/proptest/escape.rs` 等 | `tests/`配下 | パターン6の軽量プロパティ（デフォルト256ケース） |
| `tests/proptest/parser_structure.rs`, `tests/proptest/partial_cycle.rs` 等 | `tests/`配下 | パターン6の重いプロパティ（64ケースに調整） |

具体的なテストファイル構成・生成器（generator）の設計はCode Generationステージで確定する。
