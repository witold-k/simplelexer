# simplelexer

Small, dependency-free Rust lexers for preserving and inspecting simple assignment-oriented text.

The crate keeps slices into the original input instead of allocating token strings. It is primarily intended for lightweight parsing and rewriting tasks where preserving the original text layout matters.

## Components

- `QuoteLexer` splits assignment-like text into code, whitespace, assignment operators, and quoted strings.
- `StringVarLexer` splits string values while keeping nested `${...}` expressions intact.
- `Chunk` stores token kind, source slice, and byte offsets.
- `ChunkList` provides helpers for collecting and replacing simple variable assignments while preserving surrounding text.

## Example

```rust
use simplelexer::chunkkind::ChunkKind;
use simplelexer::quotelexer::QuoteLexer;

let mut lexer = QuoteLexer::new(r#"NAME = "value""#);
let chunks = lexer.lex();

assert_eq!(chunks[0].kind, ChunkKind::Code);
assert_eq!(chunks[0].text, "NAME");
assert_eq!(chunks[2].kind, ChunkKind::Assign);
assert_eq!(chunks[4].kind, ChunkKind::String);
```

## Design goals

- Keep the public API small and predictable.
- Preserve slices and byte offsets into the original input.
- Avoid external runtime dependencies.
- Handle UTF-8 input without slicing at invalid character boundaries.
- Prefer explicit, simple lexer behavior over a general parser framework.
- Consistent test layout: tests live under `tests/`, mirror the relative `src/` hierarchy where relevant, and use the source filename with a `_test.rs` suffix.

## Stability

The existing lexer and chunk APIs are intended to remain stable. Bug fixes should preserve observable behavior unless the previous behavior was invalid or unsafe.

Byte offsets in `Chunk::start` and `Chunk::end` refer to UTF-8 byte offsets in the original input.

## Development

Run the local verification suite with:

```bash
just
```

This builds the crate, runs tests, and runs Clippy.

GitHub Actions performs formatting, build, test, and Clippy checks on pushes and pull requests.

## License

Apache-2.0.
