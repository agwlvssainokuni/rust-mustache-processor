# Logical Components — core-engine

`nfr-design-patterns.md`のパターンを実現する論理コンポーネント（非公開の内部実装単位）。`domain-entities.md`（Functional Design）を拡張する。

## RenderState（新規、非公開）

Rendererの再帰呼び出し全体で共有・更新される内部状態。`nfr-design-patterns.md`パターン1〜3に対応。

```rust
pub(crate) struct RenderState<'a> {
    context_stack: Vec<&'a Value>,
    depth: usize,
    strict: bool,
}

pub(crate) const MAX_NESTING_DEPTH: usize = 100;
```

- `context_stack`: Functional Designで定義済みのコンテキストスタック（BR-4.1/BR-4.2）を`RenderState`に格納する形に整理
- `depth`: セクション・パーシャルいずれの再帰前後でインクリメント/デクリメント。`MAX_NESTING_DEPTH`超過時にエラーとする（パターン1）
- `strict`: `Mustache`エンジンの設定値をレンダリング全体で参照するためのコピー

**Step8での補正（`partial_chain`削除、`MAX_NESTING_DEPTH`値変更）**:
- 当初計画していた`partial_chain: Vec<String>`（パーシャル名チェーンによる循環検出、パターン2）は、公式spec conformanceテスト（"Recursion"）により、同名パーシャルの自己再帰がデータに基づき自然終端する正当な実装パターンであると判明したため削除した。真に無限のパーシャル再帰に対する安全装置は`depth`（`MAX_NESTING_DEPTH`）のみに一本化する
- `MAX_NESTING_DEPTH`は当初NFR Requirements/NFR Designの例示値1000としていたが、Code Generation Step4での実測（`RUST_MIN_STACK`によるWindows既定1MiB相当のスタックサイズでのテスト）により、ガード自体が発火する前に実スタックオーバーフローが発生することが判明したため、安全マージンを持たせて100に修正した（詳細は`core-engine-code-generation-plan.md`のStep4補正記録を参照）

## RenderErrorKind の拡張

`domain-entities.md`（Functional Design）で定義済みの`RenderErrorKind`（`UndefinedVariable`, `PartialNotFound`, `PartialCycleDetected`の3種）を、Code Generation時に以下のように補正する:

```rust
pub enum RenderErrorKind {
    UndefinedVariable { name: String },
    PartialNotFound { name: String },       // Step8: strictモード時のみ発生するよう意味を変更（BR-5.2）
    MaxNestingDepthExceeded { depth: usize }, // Step4で追加
    PartialParseError { name: String, message: String }, // Step4で追加（パーシャル内容自体の構文エラー用）
    // PartialCycleDetectedはStep8で削除（上記「Step8での補正」参照）
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
