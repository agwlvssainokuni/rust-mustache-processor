# rust-mustache-processor

[日本語](README.md)

A Mustache template engine implemented from scratch in Rust. Available both as a library (`mustache_processor`) and a CLI tool (`mustache`).

- Conforms to the mandatory feature set of the official [Mustache spec](https://github.com/mustache/spec) (variable interpolation, sections, inverted sections, partials, comments, delimiter changes)
- Also supports the optional modules (lambdas, template inheritance, dynamic partial names) since v0.2.0 — see "Optional Features" below
- Verified 100% conformance against the official spec conformance test suite (136 test cases across the 6 mandatory modules, 58 test cases across the 3 optional modules)
- Supports both JSON and YAML input data
- Supports concatenating multiple template files in order (similar to `cat`)
- Optional strict mode that raises an error on undefined variables

## Installation

### Using a prebuilt binary

Download the archive for your OS/architecture (`mustache-<version>-<target-triple>.tar.gz` or `.zip`) from [GitHub Releases](../../releases), extract it, and place `mustache` (`mustache.exe` on Windows) somewhere on your `PATH`.

Supported platforms: Linux (x86_64) / macOS (Apple Silicon) / Windows (x86_64)

### Building from source

```bash
git clone <this-repository>
cd rust-mustache-processor
cargo install --path .
```

This installs the `mustache` command.

## CLI Usage

```bash
mustache [OPTIONS] [TEMPLATES]...
```

### Basic usage

```bash
# Data is read from stdin by default
mustache template.tmpl < data.json

# Specify a data file explicitly
mustache template.tmpl --data data.json

# Specify an output file (defaults to stdout)
mustache template.tmpl --data data.json --output result.txt
```

### Multiple templates

When multiple template files are given, each is rendered individually in the specified order and the results are concatenated (similar to `cat`).

```bash
mustache header.tmpl body.tmpl footer.tmpl --data data.json
```

### Reading a template from stdin

```bash
cat template.tmpl | mustache --template-stdin --data data.json
```

(Combining `--template-stdin` with an unspecified `--data` — i.e. both defaulting to stdin — is an error, since both would require the same single stdin stream.)

### Partials

Partials (`{{> partial}}`) are resolved, by default, from the directory containing the template file itself (resolved per template file when multiple templates are given). Use `--partials-dir` to override this.

```bash
mustache template.tmpl --data data.json --partials-dir ./partials
```

### Strict mode

Raises an error when an undefined variable is referenced (by default, undefined variables render as an empty string).

```bash
mustache template.tmpl --data data.json --strict
```

### Data format

By default, the format is detected from the data file's extension (`.json`/`.yaml`/`.yml`). You can also specify it explicitly with `--format`.

```bash
mustache template.tmpl --data data.yaml
mustache template.tmpl --data data.txt --format yaml
```

### All options

```
Usage: mustache [OPTIONS] [TEMPLATES]...

Arguments:
  [TEMPLATES]...  Template file(s) (may be repeated; processed and concatenated in order)

Options:
      --template-stdin               Read the template from stdin (mutually exclusive with positional templates)
      --data <DATA>                  Data file (defaults to stdin if omitted)
  -o, --output <OUTPUT>              Output file (defaults to stdout if omitted)
      --partials-dir <PARTIALS_DIR>  Partial lookup directory (defaults to each template file's own directory)
      --strict                       Strict mode: error on undefined variable references
      --format <FORMAT>              Explicitly specify the data format (json or yaml)
  -h, --help                         Print help
  -V, --version                      Print version
```

## Library Usage

`Cargo.toml`:

```toml
[dependencies]
mustache_processor = { path = "../rust-mustache-processor", default-features = false }
```

With `default-features = false`, the library depends only on `serde`, excluding the CLI-only dependencies (`clap`/`serde_json`/`serde_norway`). Without it, the default `cli` feature is enabled and those are pulled in transitively as well.

```rust
use mustache_processor::Mustache;
use mustache_processor::value::{Map, Value};

let mustache = Mustache::new();
let mut data = Map::new();
data.insert("name", Value::String("World".to_string()));

let output = mustache
    .render_str("Hello, {{name}}!", &Value::Map(data))
    .unwrap();
assert_eq!(output, "Hello, World!");
```

You can also convert any type implementing `serde::Serialize` into a `Value` via `Value::from_serialize`. Partials and strict mode are configured using a builder-style API:

```rust
use mustache_processor::Mustache;
use mustache_processor::partial::DirectoryPartialResolver;

let mustache = Mustache::new()
    .with_strict(true)
    .with_partial_resolver(Box::new(DirectoryPartialResolver::new("./partials")));
```

## Optional Features

The library also supports the official Mustache spec's optional extension modules (the `~`-prefixed ones).

### Lambdas (`~lambdas`)

You can pass a `Value::Lambda` wrapping a Rust closure as the value for `{{lambda}}` (interpolation) or `{{#lambda}}...{{/lambda}}` (section) tags. Since JSON/YAML data has no way to express a function, lambdas can only be constructed **via the library API** (not from the CLI).

```rust
use std::rc::Rc;
use mustache_processor::Mustache;
use mustache_processor::value::{Map, Value};

let mustache = Mustache::new();
let mut data = Map::new();
data.insert(
    "shout",
    Value::Lambda(Rc::new(|_text: &str| "Hello!".to_string())),
);

let output = mustache.render_str("{{shout}}", &Value::Map(data)).unwrap();
assert_eq!(output, "Hello!");
```

In a section context, the lambda receives the section's raw (unrendered) body text as its argument. The return value is always re-parsed and re-rendered as a Mustache template.

### Template Inheritance (`~inheritance`)

Use `{{<parent}}...{{/parent}}` to extend a parent template, and `{{$block}}...{{/block}}` to define an overridable block with default content.

```mustache
{{<layout}}
  {{$title}}Custom title{{/title}}
{{/layout}}
```

Block "reindentation" (aligning an overridden block's leading whitespace to the position where it's expanded, rather than where it was defined) is also supported. The official spec conformance suite for `~inheritance` (27 cases) passes 27/27 (100%).

### Dynamic Partial Names (`~dynamic-names`)

`{{>* name}}` resolves `name` from the current context and uses the resulting string as the partial name.

```mustache
{{>* templateName}}
```

## Development

```bash
cargo build          # Build
cargo test           # Run all tests (unit tests, property-based tests, spec conformance tests)
cargo doc --no-deps  # Generate library API documentation
```

For detailed design decisions, see the documentation under `aidlc-docs/`.

## License

Apache License 2.0 (see `LICENSE`). Copyright agwlvssainokuni.
