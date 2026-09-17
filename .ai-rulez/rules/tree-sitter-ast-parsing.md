---
priority: critical
---

# Tree-Sitter AST Parsing

All comment identification and removal MUST use tree-sitter AST parsing. Never use regex-based comment detection.

## Core Principles

- Use the AST visitor pattern in `src/ast/visitor.rs` for tree traversal
- Define comment node types for the four languages in `src/languages/config.rs`
- Cache parsed trees when processing multiple operations on the same file
- Handle tree-sitter query errors gracefully with descriptive error messages

## Implementation Guidelines

- Keep the existing C, C++, Objective-C, and Swift comment node types aligned with their grammars
- Ensure multi-byte UTF-8 content is handled correctly during byte-offset to character-offset conversion

## Prohibited Patterns

- Never use regex to detect or match comments
- Never assume comment syntax is uniform across languages
- Never modify source code without first parsing the full AST
