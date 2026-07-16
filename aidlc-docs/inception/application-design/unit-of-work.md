# Unit of Work — rust-mustache-processor

`application-design.md`で定義したcore-engine/cliの2ユニットを、`unit-of-work-plan.md`のQ1=A（単一クレート、`src/lib.rs` + `src/main.rs`）・Q2=A（core-engine側`tests/`）の決定に基づき、実際のCargoプロジェクト構成として確定する。

## ユニット定義

### ユニット1: core-engine
- **種別**: ライブラリ（同一パッケージの`src/lib.rs`をクレートルートとする公開API）
- **責務**: Mustacheテンプレートのパース・レンダリング処理系本体。フォーマット（JSON/YAML）非依存
- **含むコンポーネント**（`components.md`参照）: Value, Parser, Template, Renderer, PartialResolver（トレイト）, DirectoryPartialResolver, Mustache（エンジン）, ParseError, RenderError
- **公開範囲**: `Value`, `Template`, `Mustache`, `PartialResolver`, `DirectoryPartialResolver`, `ParseError`, `RenderError`のみを`pub`とする。`Parser`, `Renderer`はクレート内部モジュール（非`pub`）とし、`main.rs`から直接参照できないようにする

### ユニット2: cli
- **種別**: バイナリ（同一パッケージの`src/main.rs`をエントリポイントとする、バイナリ名`mustache`）
- **責務**: コマンドライン引数解析、ファイル/標準入出力制御、JSON/YAMLデータのロード、core-engineの公開APIを用いたオーケストレーション
- **含むコンポーネント**（`components.md`参照）: CliArgs, IoController, DataLoader, CliRunner
- **公開範囲**: バイナリのためライブラリとしての公開APIはなし。core-engineの公開APIのみを利用する

## コード構成方針（Greenfield）

Q1=Aの決定に基づき、単一パッケージ内でライブラリターゲットとバイナリターゲットを分離するRust標準のlib+binパターンを採用する。

```
rust-mustache-processor/
├── Cargo.toml                  # 単一パッケージ（既存を拡張、workspace化しない）
├── LICENSE
├── src/
│   ├── lib.rs                  # core-engine 公開API（Mustache, Template, Value, PartialResolver, DirectoryPartialResolver, ParseError, RenderError の re-export）
│   ├── value.rs                # Value型
│   ├── parser.rs               # Parser（非公開）
│   ├── renderer.rs             # Renderer（非公開）
│   ├── partial.rs              # PartialResolverトレイト, DirectoryPartialResolver
│   ├── error.rs                # ParseError, RenderError, Error
│   ├── main.rs                 # cli エントリポイント（バイナリ名 mustache）
│   └── cli/
│       ├── mod.rs              # CliRunner（オーケストレーション）
│       ├── args.rs             # CliArgs
│       ├── io.rs                # IoController
│       └── data_loader.rs      # DataLoader（serde_json/serde_yaml依存はここに閉じる）
└── tests/
    ├── spec/                   # 公式mustache/specコンフォーマンステスト（JSONケース取り込み、Q2=A）
    └── proptest/               # PBTテスト（NFR-3）
```

- `src/main.rs`は`mod cli;`で`src/cli/`配下を取り込み、`cli::run()`相当のエントリポイントを呼び出すのみとする（薄いエントリポイント）
- `tests/`配下はCargoの統合テストとして`lib.rs`の公開APIのみにアクセス可能。specコンフォーマンステスト・PBTテストはいずれもcore-engineの公開API（`Mustache`, `Template`, `Value`）に対して実施する
- `[[bin]] name = "mustache"`は既存の`Cargo.toml`設定を維持

## 検証結果
- 全コンポーネント（`components.md`記載の12コンポーネント）がcore-engine/cliいずれかのユニットに過不足なく割り当てられていることを確認
- ユニット境界はApplication Designの単方向依存原則（cli → core-engineのみ）と一致し、Rustのlib+binパターンによりコンパイラレベルで強制される
