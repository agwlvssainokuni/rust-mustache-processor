# Business Logic Model — core-engine

## パース処理（Parser、非公開）

1. テンプレート文字列を先頭から走査し、現在のデリミタ（初期値`{{`/`}}`）に一致するタグを検出する
2. タグ種別を判定する: 変数（無印/`{{{}}}`/`&`）、セクション開始（`#`）、逆セクション開始（`^`）、セクション終了（`/`）、パーシャル（`>`）、コメント（`!`）、デリミタ変更（`=...=`）
3. セクション開始タグをスタックにプッシュし、対応する終了タグでポップして`Node::Section`を構築する（未対応の終了タグ・EOF到達時は`ParseErrorKind::UnbalancedSection`/`UnexpectedEof`）
4. スタンドアロンタグ判定（BR-7.1）: タグの前後が行頭からの空白のみ・行末が改行またはEOFであれば、その行の空白・改行を出力対象から除去する
5. デリミタ変更タグ（`=...=`）検出時は以降の走査で新しいデリミタを使用する（`Node`は生成しない）
6. コメントタグはそのまま読み飛ばす（`Node`は生成しない）
7. 各タグ生成時、走査位置から`SourcePosition { line, column }`を記録し`Node`に埋め込む
8. 結果として`Vec<Node>`（ルートレベルのノード列、セクションは子ノードを内包した木構造）を`Template`として返す

## レンダリング処理（Renderer、非公開）

1. `Template`のルート`Vec<Node>`を、初期コンテキストスタック`[data]`（`data: &Value`）で走査する
2. `Node::Text`はそのまま出力に追加する
3. `Node::Variable`はコンテキストスタック探索（BR-4.1/BR-4.2）で値を解決し、BR-1系ルールに従って文字列化・エスケープして出力する
4. `Node::Section`はBR-2系/BR-3系ルールに従い、真偽判定に応じて0回・1回・複数回、子ノードを（必要ならコンテキストをプッシュして）再帰的にレンダリングする
5. `Node::Partial`到達時: BR-5.1〜BR-5.5に従い`PartialResolver::resolve`を呼び出し、循環チェーンを検査した上で、得られた文字列をParserで再パースし、現在のコンテキストスタックで再帰的にレンダリングし、インデントを適用する
6. 全ノード走査後、蓄積した出力文字列を返す

## エラー伝播

- Parser内のエラーは`ParseError`として即座に呼び出し元（`Mustache::parse`）に伝播する（パース全体を中断）
- Renderer内のエラー（未定義変数・パーシャル未解決・パーシャル循環）は`RenderError`として即座に呼び出し元（`Mustache::render`）に伝播する（レンダリング全体を中断。部分的な出力は返さない）

## Testable Properties（PBT-01）

`aidlc-docs/inception/requirements/requirements.md`のNFR-3（PBT拡張機能: フル適用）に基づき、以下のテスト可能なプロパティを識別した。使用フレームワークはNFR Requirementsステージで確定する。

| コンポーネント | プロパティ | カテゴリ | 内容 |
|---|---|---|---|
| Parser | テキストのみのテンプレートはそのまま透過する | Invariant | `{{`/`}}`を含まない任意の文字列`s`について、`render(parse(s), any_value) == s` |
| Parser | セクションの入れ子構造が保存される | Induction | 深さ1のセクションでパースが成立するなら、それを子として含む深さN+1のネストしたセクションでも同様にパースが成立し、対応する`Node::Section`の子として構造化される |
| Renderer | HTMLエスケープと非エスケープの相補性 | Round-trip | 任意の文字列`s`について、BR-1.1のエスケープ規則で変換した結果をHTMLアンエスケープすると`s`に戻る（`unescape(escape(s)) == s`） |
| Renderer | セクション/逆セクションの相補性 | Invariant | 同一キー・同一コンテキストに対し、`{{#key}}A{{/key}}{{^key}}B{{/key}}`の出力は、`A`（キーが真）と`B`（キーが偽）のちょうど一方のみを含み、両方または どちらも含まない結果にはならない |
| Renderer | 配列セクションの繰り返し回数 | Invariant | `Value::Array`をコンテキストとするセクションの出力は、要素数と同じ回数だけ本体を繰り返した結果と一致する（BR-2.2） |
| Renderer（パーシャル循環検出） | 循環参照時の終端保証 | Invariant | パーシャル解決チェーンに循環を含む任意の`PartialResolver`実装に対し、レンダリングは無限再帰せず有限時間で`RenderErrorKind::PartialCycleDetected`を返す（Q4=B） |
| DirectoryPartialResolver | 同一名の解決結果の安定性 | Idempotence | ファイルシステムの状態が変化しない限り、同じ名前に対する`resolve`の呼び出しは同じ結果を返す |
| Value | — | N/A | 純粋なデータ表現であり、変換・アルゴリズムを含まないためテスト可能なプロパティは識別されない（データ構造としての等価性はexample-basedテストで十分） |
| ParseError / RenderError | — | N/A | エラー型はデータ保持のみで、業務ロジックを含まないため対象外 |

これらのプロパティはCode Generation（Planning）ステージでPBTテスト実装計画に引き継ぐ。
