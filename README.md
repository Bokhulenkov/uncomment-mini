# uncomment

This project is based on [polylint/uncomment](https://github.com/polylint/uncomment).

A Rust utility that removes comments using tree-sitter and modifies source files
in place in the specified file or directory.

```sh
cargo build --release --locked
./target/release/uncomment /path/to/project
```

Building requires Rust and a C compiler.
The compiled binary does not require Rust, the utility's source code, or network access to run.

## Supported file types

Only the following lowercase file extensions are processed:

| Extension | Grammar     |
| --------- | ----------- |
| `.h`      | C           |
| `.m`      | Objective-C |
| `.mm`     | C++         |
| `.swift`  | Swift       |

Other files are skipped. Directory traversal respects `.gitignore` rules and skips
`.git` directories and nested Git repositories. Hidden directories are processed
unless excluded by `.gitignore`. Symbolic links are not followed during traversal.

## Preservation rules

The single source of truth for preservation rules is `src/rules/preservation.rs`:

- Documentation comments starting with `///`, `/**`, or `//!` are preserved.
- Comments containing text from `PRESERVE_PATTERNS` are preserved:
  `HACK`, `WORKAROUND`, `~keep`. Matching is case-sensitive.
- `~keep` preserves a contiguous block of adjacent comments; code or a blank line
  ends the block. Example: `// ~keep: Sample comment`.
- In C, Objective-C, and C++, comments following preprocessor directives are preserved.
- TODO, FIXME, MARK, and NOTE comments are removed unless protected by one of the
  rules above. There are no default exceptions for linter directives.

To preserve additional text, update `PRESERVE_PATTERNS` and rebuild the binary.

## Running locally on macOS

Build the binary for the current Mac's architecture:

```sh
cargo build --release --locked
```

The compiled binary is `target/release/uncomment`. Copy it to a directory in `PATH`
to run it as `uncomment <path>`.

### Cleaning up after a build

To free disk space after a successful build, run the following from the utility's project root:

```sh
cp target/release/uncomment ./uncomment && cargo clean
```

This command saves the binary in the project root and, after a successful copy,
removes build artifacts from `target`, including `debug`, `release`, and intermediate files.
The source code and the copied binary remain. Run the utility after cleanup with:

```sh
./uncomment /path/to/project
```

Without copying the binary first, `cargo clean` also deletes the compiled binary from `target`.
The next build recreates the necessary files and takes longer.

## Output and errors

The command prints the number of processed and modified files.
Read, processing, traversal, or write errors cause the command to exit with a nonzero status.
Files already written are not rolled back if a later error occurs.
Subsequent runs do not rewrite files whose contents would remain unchanged.

## Running in GitLab CI/CD

Host the compiled `uncomment` binary in storage accessible to the runner and set
the CI/CD variable `UNCOMMENT_BINARY_URL` to a direct download URL for that file.
The binary must be built for the runner's operating system and architecture.

In `.gitlab-ci.yml`, add these commands to `before_script` in the existing application
build job. If `before_script` is already defined, append the commands to it:

```yaml
before_script:
  - mkdir -p "$CI_PROJECT_DIR/.ci-tools"
  - curl --fail --location "$UNCOMMENT_BINARY_URL" --output "$CI_PROJECT_DIR/.ci-tools/uncomment"
  - chmod +x "$CI_PROJECT_DIR/.ci-tools/uncomment"
  - '"$CI_PROJECT_DIR/.ci-tools/uncomment" "$CI_PROJECT_DIR"'
```

`CI_PROJECT_DIR` is the application's working directory in the job. The utility modifies
the source files there, then `script` in the same job builds the application from the cleaned files.
If downloading or processing fails, the job stops before the build starts.
