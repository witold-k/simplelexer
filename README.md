# simplelexer

Small, dependency-free Rust lexers for preserving and inspecting simple assignment-oriented text.

The crate keeps slices into the original input instead of allocating token strings. It is primarily intended for lightweight parsing and rewriting tasks where preserving the original text layout matters.

## Components

- `QuoteLexer` splits assignment-like text into code, whitespace, assignments, other recognized operators, and quoted strings.
- `StringVarLexer` splits string values while keeping nested `${...}` expressions intact.
- `Chunk` stores token kind, source slice, and UTF-8 byte offsets into the original lexer input.
- `ChunkList` is an owning, editable representation for collecting and replacing simple variable assignments while preserving surrounding text.

## Example

```rust
use simplelexer::chunkkind::ChunkKind;
use simplelexer::quotelexer::QuoteLexer;

let lexer = QuoteLexer::new(r#"NAME = "value""#);
let chunks = lexer.lex();

assert_eq!(chunks[0].kind, ChunkKind::Code);
assert_eq!(chunks[0].text, "NAME");
assert_eq!(chunks[2].kind, ChunkKind::Assign);
assert_eq!(chunks[4].kind, ChunkKind::String);
```

## Design goals

- Keep the public API small and predictable after the current hardening pass.
- Keep lexer output zero-copy and preserve byte offsets into the original input.
- Keep editable `ChunkList` data owned so edits are independent of input lifetimes.
- Avoid external runtime dependencies.
- Handle UTF-8 input without slicing at invalid character boundaries.
- Prefer explicit, simple lexer behavior over a general parser framework.
- Consistent test layout: tests live under `tests/`, mirror the relative `src/` hierarchy where relevant, and use the source filename with a `_test.rs` suffix.

## API model

`QuoteLexer` and `StringVarLexer` return borrowed `Chunk<'a>` values. This keeps lexing allocation-free for token text. Both lexers are reusable: calling `lex()` repeatedly returns the same result. `Chunk::start` / `Chunk::end` are UTF-8 byte offsets into the original input, including when `StringVarLexer` trims whitespace or matching outer quotes.

`ChunkKind::Assignment` is reserved for assignment forms; comparison, arrow, increment/decrement, and related recognized operators are reported as `ChunkKind::Operator`.

`ChunkList` deliberately owns its `OwnedChunk` strings. Editing therefore does not impose source-text lifetimes on replacement values. Use `ChunkList::from_text` to create an editable representation and `ChunkList::chunks` for read-only chunk access.

`ChunkList::replace_value` returns `true` when an existing assignment was replaced and `false` when a new assignment had to be appended.

The intent of this hardening pass is to settle these ownership boundaries now so future users do not need an interface migration for ordinary editing.

## Development

Run the local verification suite with:

```bash
just
```

This builds the crate, runs tests, and runs Clippy.

GitHub Actions performs formatting, build, test, and Clippy checks on pushes and pull requests.

## License

Apache-2.0.
