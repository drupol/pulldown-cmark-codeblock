![GitHub stars][GitHub stars]
[![Crates.io Version][Crates.io Version]][pulldown-cmark-codeblock crates]
[![Crates.io License][Crates.io License]][pulldown-cmark-codeblock crates]
[![Donate!][Donate!]][sponsor link]

# pulldown-cmark-codeblock

Extract Markdown code blocks from Markdown documents parsed with
[`pulldown-cmark`](https://crates.io/crates/pulldown-cmark).

`pulldown-cmark` already exposes the fenced code block info string through
`Event::Start(Tag::CodeBlock(CodeBlockKind::Fenced(info_string)))`. This crate
builds on top of that lower-level event stream and returns complete,
ready-to-use code block records.

- fenced or indented block kind
- language parsed from the first info string word
- raw info string
- remaining attributes as a raw string or token iterator
- code block source text
- byte range covering the whole block
- zero-based line range covering the whole block
- indentation before the opening marker

````rust
use pulldown_cmark_codeblock::{code_blocks, CodeBlockKind};

let markdown = "# Title\n\n```rust runnable key=value\nfn main() {}\n```\n";
let block = code_blocks(markdown).next().unwrap();

assert!(matches!(block.kind, CodeBlockKind::Fenced(_)));
assert_eq!(block.language.as_deref(), Some("rust"));
assert_eq!(block.info_string, "rust runnable key=value");
assert_eq!(block.attributes.as_deref(), Some("runnable key=value"));
assert_eq!(block.attributes().collect::<Vec<_>>(), ["runnable", "key=value"]);
assert_eq!(block.source, "fn main() {}\n");
assert_eq!(block.line_range, 2..5);
````

The detailed API documentation is written in `src/lib.rs` with crate-level
`//!` rustdoc comments, so it is generated directly by `cargo doc` and
published with the crate documentation.

[GitHub stars]: https://img.shields.io/github/stars/drupol/pulldown-cmark-codeblock.svg?style=flat-square
[Donate!]: https://img.shields.io/badge/Sponsor-Github-brightgreen.svg?style=flat-square
[sponsor link]: https://github.com/sponsors/drupol
[Crates.io License]: https://img.shields.io/crates/l/pulldown-cmark-codeblock?style=flat-square
[Crates.io Version]: https://img.shields.io/crates/v/pulldown-cmark-codeblock?style=flat-square
[pulldown-cmark-codeblock crates]: https://crates.io/crates/pulldown-cmark-codeblock
