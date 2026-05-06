//! Extract Markdown code blocks from Markdown documents parsed with
//! [`pulldown-cmark`](https://crates.io/crates/pulldown-cmark).
//!
//! `pulldown-cmark` already exposes the fenced code block info string through
//! `Event::Start(Tag::CodeBlock(CodeBlockKind::Fenced(info_string)))`. This
//! crate builds on top of that lower-level event stream and returns complete,
//! ready-to-use code block records:
//!
//! - fenced or indented block kind
//! - language parsed from the first info string word
//! - raw info string
//! - remaining attributes as a raw string or token iterator
//! - code block source text
//! - byte range covering the whole block
//! - zero-based line range covering the whole block
//! - indentation before the opening marker
//!
//! # Example
//!
//! ````rust
//! use pulldown_cmark_codeblock::{code_blocks, CodeBlockKind};
//!
//! let markdown = "# Title\n\n```rust runnable key=value\nfn main() {}\n```\n";
//! let block = code_blocks(markdown).next().unwrap();
//!
//! assert!(matches!(block.kind, CodeBlockKind::Fenced(_)));
//! assert_eq!(block.language.as_deref(), Some("rust"));
//! assert_eq!(block.info_string, "rust runnable key=value");
//! assert_eq!(block.attributes.as_deref(), Some("runnable key=value"));
//! assert_eq!(block.attributes().collect::<Vec<_>>(), ["runnable", "key=value"]);
//! assert_eq!(block.source, "fn main() {}\n");
//! assert_eq!(block.line_range, 2..5);
//! ````
//!
//! # API
//!
//! Use [`code_blocks`] for the concise iterator API:
//!
//! ````rust
//! use pulldown_cmark_codeblock::code_blocks;
//!
//! let markdown = "Before\n\n```rust\nfn main() {}\n```\n";
//! let blocks = code_blocks(markdown).collect::<Vec<_>>();
//!
//! assert_eq!(blocks.len(), 1);
//! assert_eq!(blocks[0].language.as_deref(), Some("rust"));
//! ````
//!
//! Use [`CodeBlockExtractor::from_markdown`] when you prefer constructing the
//! iterator explicitly:
//!
//! ````rust
//! use pulldown_cmark_codeblock::CodeBlockExtractor;
//!
//! let markdown = "```rust\nfn main() {}\n```\n";
//! let blocks = CodeBlockExtractor::from_markdown(markdown).collect::<Vec<_>>();
//!
//! assert_eq!(blocks[0].source, "fn main() {}\n");
//! ````
//!
//! Each extracted [`CodeBlock`] exposes:
//!
//! - [`CodeBlock::kind`]: [`CodeBlockKind::Fenced`] or
//!   [`CodeBlockKind::Indented`]
//! - [`CodeBlock::language`]: first info string word for fenced blocks
//! - [`CodeBlock::info_string`]: complete fenced block info string
//! - [`CodeBlock::attributes`]: remaining info string after the language
//! - [`CodeBlock::source`]: code block body
//! - [`CodeBlock::byte_range`]: byte range covering opening marker, body, and
//!   closing marker
//! - [`CodeBlock::line_range`]: zero-based line range covering the whole block
//! - [`CodeBlock::indent`]: whitespace indentation before the opening marker
//!
//! It also provides helper methods:
//!
//! - [`CodeBlock::is_fenced`]
//! - [`CodeBlock::has_info_word`]
//! - [`CodeBlock::attributes`]
//! - [`CodeBlock::has_attribute`]
//!
//! ````rust
//! use pulldown_cmark_codeblock::code_blocks;
//!
//! let markdown = "```rust a b c\nfn main() {}\n```\n";
//! let block = code_blocks(markdown).next().unwrap();
//!
//! assert!(block.is_fenced());
//! assert!(block.has_info_word("rust"));
//! assert!(block.has_attribute("b"));
//! assert_eq!(block.attributes().collect::<Vec<_>>(), ["a", "b", "c"]);
//! ````
//!
//! Indented code blocks are also returned. They do not have an info string,
//! language, or attributes.
//!
//! ```rust
//! use pulldown_cmark_codeblock::{code_blocks, CodeBlockKind};
//!
//! let markdown = "Before\n\n    indented\n\nAfter\n";
//! let block = code_blocks(markdown).next().unwrap();
//!
//! assert!(matches!(block.kind, CodeBlockKind::Indented));
//! assert_eq!(block.language, None);
//! assert_eq!(block.info_string, "");
//! assert_eq!(block.attributes, None);
//! assert_eq!(block.source, "indented\n");
//! ```
#![forbid(unsafe_code)]
#![warn(missing_docs, future_incompatible, rust_2018_idioms)]

use std::ops::Range;

use pulldown_cmark::{DefaultBrokenLinkCallback, Event, OffsetIter, Parser, Tag, TagEnd};

pub use pulldown_cmark::CodeBlockKind;

/// A code block extracted from a Markdown document.
#[derive(Debug, Clone, PartialEq)]
pub struct CodeBlock {
    /// The code block kind.
    pub kind: CodeBlockKind<'static>,
    /// First word from the fenced code block info string.
    ///
    /// This is [`None`] for indented code blocks and for fenced code blocks
    /// without an info string.
    pub language: Option<String>,
    /// Complete fenced code block info string, without the opening backticks.
    ///
    /// This is empty for indented code blocks.
    pub info_string: String,
    /// Remaining info string words after the language, if present.
    pub attributes: Option<String>,
    /// Code block body.
    pub source: String,
    /// Byte range covering the opening marker, body, and closing marker.
    pub byte_range: Range<usize>,
    /// Zero-based line range covering the opening marker, body, and closing marker.
    pub line_range: Range<usize>,
    /// Whitespace indentation before the opening marker.
    pub indent: usize,
}

impl CodeBlock {
    /// Returns true for fenced code blocks.
    #[must_use]
    pub fn is_fenced(&self) -> bool {
        self.kind.is_fenced()
    }

    /// Returns true when the full info string contains `word` as a whitespace-separated token.
    #[must_use]
    pub fn has_info_word(&self, word: &str) -> bool {
        self.info_string
            .split_whitespace()
            .any(|token| token == word)
    }

    /// Returns fenced code block attributes as whitespace-separated tokens.
    ///
    /// For ```` ```rust a b c ````, this yields `a`, `b`, and `c`.
    pub fn attributes(&self) -> impl Iterator<Item = &str> {
        self.attributes.as_deref().unwrap_or("").split_whitespace()
    }

    /// Returns true when the fenced code block attributes contain `attribute`.
    #[must_use]
    pub fn has_attribute(&self, attribute: &str) -> bool {
        self.attributes().any(|token| token == attribute)
    }
}

/// Iterator over Markdown code blocks.
pub struct CodeBlockExtractor<'a> {
    parser: OffsetIter<'a, DefaultBrokenLinkCallback>,
    markdown: &'a str,
}

impl<'a> CodeBlockExtractor<'a> {
    /// Creates an extractor from Markdown source.
    #[must_use]
    pub fn from_markdown(markdown: &'a str) -> Self {
        Self {
            parser: Parser::new(markdown).into_offset_iter(),
            markdown,
        }
    }
}

impl Iterator for CodeBlockExtractor<'_> {
    type Item = CodeBlock;

    fn next(&mut self) -> Option<Self::Item> {
        while let Some((event, range)) = self.parser.next() {
            if let Event::Start(Tag::CodeBlock(kind)) = event {
                return Some(self.collect_code_block(kind, range));
            }
        }

        None
    }
}

impl CodeBlockExtractor<'_> {
    fn collect_code_block(
        &mut self,
        kind: CodeBlockKind<'_>,
        start_range: Range<usize>,
    ) -> CodeBlock {
        let mut source = String::new();
        let mut end_offset = start_range.end;

        for (event, range) in &mut self.parser {
            match event {
                Event::Text(text) => {
                    source.push_str(&text);
                    end_offset = range.end;
                }
                Event::End(TagEnd::CodeBlock) => {
                    end_offset = range.end;
                    break;
                }
                _ => {}
            }
        }

        let kind = kind.into_static();
        let info_string = match &kind {
            CodeBlockKind::Fenced(info_string) => info_string.to_string(),
            CodeBlockKind::Indented => String::new(),
        };
        let (language, attributes) = parse_info_string(&info_string);
        let indent = self
            .markdown
            .get(..start_range.start)
            .and_then(|source| source.lines().last())
            .unwrap_or("")
            .chars()
            .take_while(|character| character.is_whitespace())
            .count();

        CodeBlock {
            kind,
            language,
            info_string,
            attributes,
            source,
            byte_range: start_range.start..end_offset,
            line_range: line_number(self.markdown, start_range.start)
                .saturating_sub(usize::from(indent > 0))
                ..line_number(self.markdown, end_offset),
            indent,
        }
    }
}

/// Returns an iterator over Markdown code blocks.
#[must_use]
pub fn code_blocks(markdown: &str) -> CodeBlockExtractor<'_> {
    CodeBlockExtractor::from_markdown(markdown)
}

fn parse_info_string(info_string: &str) -> (Option<String>, Option<String>) {
    let trimmed = info_string.trim();

    if trimmed.is_empty() {
        return (None, None);
    }

    match trimmed.split_once(char::is_whitespace) {
        Some((language, attributes)) => {
            let attributes = attributes.trim();
            (
                Some(language.to_string()),
                (!attributes.is_empty()).then(|| attributes.to_string()),
            )
        }
        None => (Some(trimmed.to_string()), None),
    }
}

fn line_number(markdown: &str, offset: usize) -> usize {
    markdown[..offset].lines().count()
}

#[cfg(test)]
mod tests {
    use super::{CodeBlockExtractor, CodeBlockKind, code_blocks};

    #[test]
    fn extracts_fenced_code_blocks() {
        let markdown = "# Title\n\n```rust mdcr-skip key=value\nfn main() {}\n```\n";

        let blocks = CodeBlockExtractor::from_markdown(markdown).collect::<Vec<_>>();

        assert_eq!(blocks.len(), 1);
        assert!(matches!(blocks[0].kind, CodeBlockKind::Fenced(_)));
        assert_eq!(blocks[0].language.as_deref(), Some("rust"));
        assert_eq!(blocks[0].attributes.as_deref(), Some("mdcr-skip key=value"));
        assert_eq!(blocks[0].info_string, "rust mdcr-skip key=value");
        assert_eq!(blocks[0].source, "fn main() {}\n");
        assert_eq!(blocks[0].line_range, 2..5);
        assert!(blocks[0].has_info_word("mdcr-skip"));
        assert!(blocks[0].has_attribute("mdcr-skip"));
    }

    #[test]
    fn extracts_fenced_code_block_attributes() {
        let markdown = "```rust a b c\nfn main() {}\n```\n";

        let blocks = code_blocks(markdown).collect::<Vec<_>>();

        assert_eq!(blocks[0].language.as_deref(), Some("rust"));
        assert_eq!(blocks[0].attributes.as_deref(), Some("a b c"));
        assert_eq!(blocks[0].attributes().collect::<Vec<_>>(), ["a", "b", "c"]);
    }

    #[test]
    fn extracts_indented_code_blocks() {
        let markdown = "Before\n\n    indented\n\nAfter\n";

        let blocks = code_blocks(markdown).collect::<Vec<_>>();

        assert_eq!(blocks.len(), 1);
        assert!(matches!(blocks[0].kind, CodeBlockKind::Indented));
        assert_eq!(blocks[0].language, None);
        assert_eq!(blocks[0].source, "indented\n");
    }
}
