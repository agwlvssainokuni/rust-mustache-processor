# Logical Components — cli

`nfr-design-patterns.md`のパターンを実現する論理コンポーネント。`domain-entities.md`（Functional Design）を拡張する。

## CliError の`From`実装（パターン4）

```rust
impl From<CliArgsError> for CliError {
    fn from(e: CliArgsError) -> Self {
        CliError::Args(e)
    }
}

impl From<IoError> for CliError {
    fn from(e: IoError) -> Self {
        CliError::Io(e)
    }
}

impl From<DataLoaderError> for CliError {
    fn from(e: DataLoaderError) -> Self {
        CliError::DataLoader(e)
    }
}

impl From<mustache_processor::error::Error> for CliError {
    fn from(e: mustache_processor::error::Error) -> Self {
        CliError::Mustache(e)
    }
}

impl std::fmt::Display for CliError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CliError::Args(e) => write!(f, "{e}"),       // CliArgsErrorもDisplayを実装
            CliError::Io(e) => write!(f, "{e}"),         // IoErrorも同様
            CliError::DataLoader(e) => write!(f, "{e}"), // DataLoaderErrorも同様
            CliError::Mustache(e) => write!(f, "{e}"),   // 既存のError::Display委譲
        }
    }
}
```

## run_inner / run（パターン1）

```rust
pub(crate) fn run_inner(argv: &[String]) -> Result<(), CliError> {
    let args = CliArgs::parse_args(argv)?;

    let templates = IoController::read_templates(&args)?;
    let data_raw = IoController::read_data(&args)?;
    let format = DataLoader::detect_format(&args)?;
    let data = DataLoader::load(&data_raw, format)?;

    let mut out = String::new();
    for loaded in &templates {
        // パターン3: テンプレートごとにパーシャルディレクトリを解決し、エンジンを構築する
        let partials_dir = IoController::resolve_partials_dir(&args, &loaded.source);
        let mustache = Mustache::new()
            .with_strict(args.strict)
            .with_partial_resolver(Box::new(DirectoryPartialResolver::new(partials_dir)));

        // パターン2: 1つでも失敗したら即座に中断し、outは書き出さない
        let rendered = mustache.render_str(&loaded.content, &data)?;
        out.push_str(&rendered);
    }

    IoController::write_output(&args, &out)?;
    Ok(())
}

/// `component-methods.md`が要求する公開シグネチャ。`run_inner`の薄いラッパー（パターン1）。
pub(crate) fn run(argv: &[String]) -> std::process::ExitCode {
    match run_inner(argv) {
        Ok(()) => std::process::ExitCode::from(0),
        Err(e) => {
            eprintln!("mustache: {e}"); // BR-7.2
            std::process::ExitCode::from(1) // BR-7.1
        }
    }
}
```

- `run_inner`が`?`演算子で各エラー型を`CliError`へ自動変換できるのは、上記`From`実装群による
- `render_str`は`mustache_processor::error::Error`を返すため、`From<mustache_processor::error::Error> for CliError`により`?`でそのまま伝播する
