---
priority: high
---

# Comment Preservation Rules

uncomment supports selective comment preservation. Certain comments must not be removed even during full comment stripping.

## Preserved Comment Types

- **TODO/FIXME markers**: Comments containing `TODO` or `FIXME`, unless explicitly requested for removal
- **C-family and Swift directives**: Comments matching configured preservation patterns, such as `NOLINT` or `swiftlint:`
- **Keep marker**: Comments containing `~keep` are explicitly marked for preservation
- **Documentation comments**: Comments such as `///` and `/** */`

## Configuration

- Built-in preservation patterns are defined in `src/rules/preservation.rs`
- Users can add preserve patterns in `.uncommentrc.toml` under `[languages.<name>].preserve_patterns` for the four supported languages
- The preservation rules engine lives in `src/rules/`

## Guidelines

- Test preservation with edge cases: nested comments, comments at start/end of file, comments on the same line as code
- The `~keep` marker should work across all four supported languages
