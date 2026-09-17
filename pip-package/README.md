# uncomment

A Rust-based CLI that removes comments from C, C++, Objective-C, and Swift source using tree-sitter.

Supported extensions: `.c`, `.h`, `.cpp`, `.cxx`, `.cc`, `.c++`, `.hpp`, `.hxx`, `.hh`, `.h++`, `.m`, and `.swift`. `.mm` is unsupported. TOML configuration can adjust preservation behavior for these languages; it cannot register other languages or extensions.

## Installation

```bash
pip install uncomment
```

This package installs or invokes the Rust binary. The package implementation language does not affect which source files the binary processes.

## Usage

```bash
uncomment main.c
uncomment --dry-run src/
uncomment --remove-doc api.swift
```

See the [project documentation](https://github.com/Goldziher/uncomment) for options and configuration.
