doc-parser
=========

Rust library to parse Markdown documents into card definitions for Anki-like workflows.

Status: initial scaffold. Implements simple line-based parsing for `->` and `<->`.

Planned next steps:
- Switch to AST-based parsing (`pulldown-cmark` walker)
- Preserve nested list context in question text
- Expand test coverage and add property tests
