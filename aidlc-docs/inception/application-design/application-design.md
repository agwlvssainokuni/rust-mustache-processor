# Application Design — rust-mustache-processor

`requirements.md`および`application-design-plan.md`（設計判断Q1=C, Q2=B, Q3=A, Q4=B, Q5=B）を踏まえたアプリケーション設計。詳細は各分割ドキュメントを参照。

- コンポーネント定義: [components.md](components.md)
- メソッドシグネチャ（概要）: [component-methods.md](component-methods.md)
- サービス/オーケストレーション: [services.md](services.md)
- 依存関係: [component-dependency.md](component-dependency.md)

## ユニット構成

| ユニット | 種別 | 概要 |
|---|---|---|
| core-engine | ライブラリ | Mustache処理エンジン本体（Value, Parser, Template, Renderer, PartialResolver, DirectoryPartialResolver, Mustache, エラー型） |
| cli | バイナリ（`mustache`） | core-engineの薄いラッパー（CliArgs, IoController, DataLoader, CliRunner） |

## 設計判断サマリー

| 項目 | 決定 | 詳細 |
|---|---|---|
| 公開API形状 | エンジン設定オブジェクト型 | `Mustache`エンジンが設定を保持し、`.parse()`と`.render()`を提供。加えて一括処理用`.render_str()`も提供 |
| パーシャル解決の抽象化 | トレイト方式 | `PartialResolver`トレイトを定義し、`DirectoryPartialResolver`を標準実装として同梱 |
| パーシャル未解決時のエラータイミング | 遅延評価 | レンダリング中に実際にパーシャルタグへ到達した時点で解決・エラー判定を行う |
| JSON/YAMLパースの配置 | cli限定 | core-engineはフォーマット非依存の`Value`型のみを受け取る。JSON/YAML変換はcliの`DataLoader`が担う |
| エラー型の粒度 | パース/レンダリングで分離 | `ParseError`と`RenderError`を別型とし、一括処理用に両者を包む`Error`型も用意 |

## コンポーネント一覧（概要）

**core-engine**: Value, Parser, Template, Renderer, PartialResolver（トレイト）, DirectoryPartialResolver, Mustache（エンジン）, ParseError/RenderError

**cli**: CliArgs, IoController, DataLoader, CliRunner

詳細な責務・インターフェースは[components.md](components.md)を参照。

## アーキテクチャ原則

1. **単方向依存**: cli → core-engine のみ。core-engineはcliを一切知らない
2. **公開境界の限定**: core-engineの内部モジュール（Parser, Renderer）は非公開とし、`Mustache`エンジン経由でのみ利用可能にする
3. **フォーマット非依存**: core-engineはJSON/YAML等のデータフォーマットに依存しない（`serde::Serialize`境界のみ）
4. **遅延評価の一貫性**: パーシャル解決を含め、Mustache仕様の「評価されないセクションは処理しない」という原則をレンダリング全体で維持する

## 検証結果

- **完全性**: 要件定義のFR-1〜FR-7（FR-8の著作権表記は横断的要件のためCode Generationで適用）に対応するコンポーネントが存在することを確認
- **一貫性**: 5つの設計判断（Q1〜Q5）が相互に矛盾なく整合していることを確認（例: Q1のエンジン設定オブジェクト型とQ5のエラー型分離は自然に対応）
- **Functional Designへの引き継ぎ事項**: 各メソッドの詳細な業務ルール（真偽判定の厳密な条件、エスケープ処理の細部、デリミタ変更のスコープ規則等）およびPBT-01（テスト可能なプロパティの識別）はFunctional Design（per-unit）で定義する