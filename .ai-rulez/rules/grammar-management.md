---
priority: high
---

# Grammar Management

Tree-sitter grammars are the foundation of uncomment's parsing capabilities. The processor uses `tree-sitter-language-pack` for the four built-in languages.

## Built-in Grammars

- Defined in `src/languages/config.rs` and registered in `src/languages/registry.rs`
- Selected from file paths by the built-in registry before parsing
- Limited to C, C++, Objective-C, and Swift; `.mm` is not registered

## Guidelines

- Keep parser definitions, registry extensions, CLI selection, and fixtures consistent
- Report grammar lookup and parsing failures clearly
