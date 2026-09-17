<div align="center">

<img src="assets/banner.svg" alt="uncomment — strip the noise, keep the code" width="820">

**Strip the noise. Keep the code.**

uncomment removes comments from source code using tree-sitter's AST — so it is 100% accurate and
**never** touches comment-like text inside strings. It keeps what matters by default (TODO/FIXME,
docs, and linting directives) in **C, C++, Objective-C, and Swift**, with parallel processing and a safe dry-run
mode.

AST-accurate&nbsp;·&nbsp;4 languages&nbsp;·&nbsp;zero false positives&nbsp;·&nbsp;smart preservation&nbsp;·&nbsp;parallel&nbsp;·&nbsp;dry-run

[![crates.io](https://img.shields.io/crates/v/uncomment?style=flat-square&color=2dd4bf)](https://crates.io/crates/uncomment)
[![npm](https://img.shields.io/npm/v/uncomment-cli?style=flat-square&color=2dd4bf&label=npm)](https://www.npmjs.com/package/uncomment-cli)
[![PyPI](https://img.shields.io/pypi/v/uncomment?style=flat-square&color=2dd4bf)](https://pypi.org/project/uncomment/)
[![CI](https://img.shields.io/github/actions/workflow/status/Goldziher/uncomment/ci.yml?style=flat-square&label=CI)](https://github.com/Goldziher/uncomment/actions)
[![License: MIT](https://img.shields.io/badge/License-MIT-2dd4bf?style=flat-square)](./LICENSE)
[![Sponsor](https://img.shields.io/badge/Sponsor-%E2%9D%A4-2dd4bf?style=flat-square&logo=github-sponsors)](https://github.com/sponsors/Goldziher)

[Install](#installation)&nbsp;·&nbsp;[Features](#features)&nbsp;·&nbsp;[Usage](#usage)&nbsp;·&nbsp;[Configuration](#configuration)&nbsp;·&nbsp;[How it works](#how-it-works)&nbsp;·&nbsp;[Contributing](#contributing)

</div>

---

## Why uncomment

Regex-based comment strippers guess. They delete a `//` inside a string literal, mangle a URL in a
comment, or leave a linting directive your CI depends on. uncomment doesn't guess: it parses your
code into a real syntax tree and removes only the nodes that are genuinely comments.

Originally built to clean up AI-generated code drowning in explanatory comments, it processes C-family and Swift source files.

## Features

- **100% accurate** — tree-sitter AST parsing identifies comments structurally, not by pattern matching
- **No false positives** — never removes comment-like content from strings
- **Smart preservation** — keeps TODO/FIXME, docs, and relevant linting directives by default
- **4 languages** — C, C++, Objective-C, and Swift via tree-sitter grammars
- **Parallel** — multi-threaded processing that scales across cores
- **Safe** — dry-run mode with line-by-line diffs previews every change before you write
- **Configurable** — hierarchical TOML config with a smart `init` command
- **Built-in benchmarking** — optional performance analysis and profiling tools

## Installation

| Channel                | Command                                            |
| ---------------------- | -------------------------------------------------- |
| Homebrew (macOS/Linux) | `brew tap goldziher/tap && brew install uncomment` |
| Cargo (Rust)           | `cargo install uncomment`                          |
| npm (Node.js)          | `npm install -g uncomment-cli`                     |
| pip (Python)           | `pip install uncomment`                            |

Prefer prebuilt binaries? [`cargo binstall uncomment`](https://github.com/cargo-bins/cargo-binstall) downloads a
release archive instead of compiling from source.

Run without installing:

```bash
npx -y uncomment-cli@latest .
uvx uncomment .
```

Add `--dry-run` to preview changes before writing.

<details>
<summary><b>Build from source</b></summary>

```bash
git clone https://github.com/Goldziher/uncomment.git
cd uncomment
cargo install --path .
```

Requires Rust 1.70+. npm and pip packages download pre-built binaries automatically.

</details>

## Quick Start

```bash
# Generate a configuration file tuned to your project
uncomment init

# Remove comments from a directory
uncomment src/

# Preview changes as a diff, write nothing
uncomment src/ --dry-run --diff
```

## Usage

```bash
# Single file
uncomment file.c

# Multiple files / globs
uncomment src/*.cpp

# Also strip documentation comments
uncomment --remove-doc file.c

# Also remove TODO and FIXME comments (preserved by default)
uncomment --remove-todo --remove-fixme file.c

# Add custom patterns to preserve
uncomment --ignore "HACK" --ignore "WARNING" file.c

# Process an entire tree with all CPU cores
uncomment . -j 0
```

Run `uncomment --help` for the full, grouped list of options.

<details>
<summary><b>Configuring with <code>init</code></b></summary>

The `init` command detects the languages in your project and writes a matching `.uncommentrc.toml`:

```bash
# Smart detection — includes detected languages, or all four if none are found
uncomment init

# All four supported languages
uncomment init --comprehensive

# Interactive selection
uncomment init --interactive

# Custom output location / overwrite
uncomment init --output config/uncomment.toml --force
```

</details>

<details>
<summary><b>Optional benchmarking tools</b></summary>

Development binaries for benchmarking and profiling are gated behind the `bench-tools` feature so
they are not installed for regular users:

```bash
# Install with extras
cargo install uncomment --features bench-tools

# Or run locally
cargo run --release --features bench-tools --bin benchmark -- --target /path/to/repo --iterations 3
cargo run --release --features bench-tools --bin profile -- /path/to/repo
```

</details>

## Supported Languages

uncomment processes C (`.c`, `.h`), C++ (`.cpp`, `.cxx`, `.cc`, `.c++`, `.hpp`, `.hxx`, `.hh`, `.h++`), Objective-C (`.m`), and Swift (`.swift`). The `.h` extension uses the C grammar, while `.m` uses Objective-C. Objective-C++ (`.mm`) is unsupported.

CLI file selection and the processor use the same built-in extension registry. TOML language settings can adjust preservation options for those four languages, but cannot register another language or extension.

## Preservation Rules

Certain comments are **never removed by default** — uncomment protects the ones your tooling and
teammates rely on.

**Always preserved:**

- Comments containing `~keep`
- `TODO` (unless `--remove-todo`), `FIXME` (unless `--remove-fixme`)
- Documentation comments (unless `--remove-doc`)

### The `~keep` marker

Put `~keep` on a `//`-style comment, not on a `///` or `/** */` doc comment. `~keep` is
plain comment text, so a marker written inside a doc comment is republished by every tool
that consumes documentation comments — documentation generators and editor hover text.

A marker on its own line protects the comment directly beneath it, which keeps it out of
anything rendered:

```c
// ~keep
/** Parent element ID for hierarchical relationships. */
int parent_id;
```

The marker also extends across a contiguous run of comments, so one `~keep` protects a
whole multi-line block. A blank line or any code between comments ends the run.

Inside a doc comment the marker is usually redundant, since doc comments are preserved
anyway unless `--remove-doc` is set. uncomment strips such a marker from the doc text and
reports it (`stripped 2 redundant ~keep markers`), leaving the comment itself untouched.
Two cases are left alone: a marker is kept when `--remove-doc` is set, because there it is
the only thing protecting that doc comment, and prose *about* the marker is never rewritten
— a line containing backticks, or a commented-out code sample, is read as documentation
rather than as a directive.

<details>
<summary><b>Linting &amp; formatter directives</b></summary>

C, C++, and Objective-C preserve common directives such as `NOLINT`, `#pragma`, and `clang-format off/on`. Swift preserves `swiftlint:` and `MARK:`. The `--no-default-ignores` flag disables these default patterns.

</details>

## Configuration

uncomment reads a hierarchical TOML configuration, merged highest-to-lowest precedence:

1. Command-line flags
2. Local `.uncommentrc.toml` (closest to the file being processed wins)
3. Global `~/.config/uncomment/config.toml`
4. Built-in defaults

```toml
[global]
remove_todos = false
remove_fixme = false
remove_docs = false
preserve_patterns = ["IMPORTANT", "NOTE", "WARNING"]
use_default_ignores = true
respect_gitignore = true

[languages.swift]
preserve_patterns = ["swiftlint:"]

[patterns."tests/**/*.swift"]
remove_todos = false
```

Language names and extensions remain fixed by the built-in registry; configuration entries only supply preservation options for those languages.

## How It Works

Unlike regex-based tools, uncomment builds a proper Abstract Syntax Tree of your code with
tree-sitter, so it distinguishes:

- Real comments vs comment-like content in strings
- Documentation comments vs regular comments
- Inline comments vs standalone comments
- Language-specific metadata that must be preserved

The pipeline is modular: a **language registry** (four built-ins) feeds an
**AST visitor** that finds comment nodes, a **preservation engine** decides what to keep, and an
**output generator** emits clean code.

## Git Hooks

<details>
<summary><b>pre-commit</b></summary>

```yaml
repos:
  - repo: https://github.com/Goldziher/uncomment
    rev: v3.5.0
    hooks:
      - id: uncomment
```

</details>

<details>
<summary><b>Lefthook</b></summary>

```yaml
pre-commit:
  commands:
    uncomment:
      run: uncomment {staged_files}
      stage_fixed: true
```

</details>

## Performance

AST parsing costs a little more than regex, but the tool is fast and scales well with threads.

- Small files (<1000 lines): ~20-30ms
- Large files (>10000 lines): ~100-200ms

| Threads | Files/second | Speedup |
| ------- | ------------ | ------- |
| 1       | 1,500        | 1.0×    |
| 4       | 3,900        | 2.6×    |
| 8       | 5,100        | 3.4×    |

*Benchmark results are historical.* Measure your own workload with the built-in `benchmark` and `profile` tools (see [optional benchmarking tools](#usage)).

## Development

```bash
cargo build              # Debug build
cargo test               # Run the test suite
cargo clippy             # Lint
cargo fmt --all          # Format
```

See [`CONTRIBUTING.md`](./CONTRIBUTING.md) for local development, automation hooks, and release
procedures.

## Contributing

Issues and pull requests are welcome. If uncomment is useful to you, consider
[sponsoring development](https://github.com/sponsors/Goldziher) — it helps keep the project
maintained for the community.

## License

[MIT](./LICENSE)
