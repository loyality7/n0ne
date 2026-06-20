# Contributing to n0ne

Thank you for your interest in contributing to the `n0ne` programming language! 

Here is a guide to help you get started.

## Project Structure

- `compiler/` - The core compiler source code (lexer, parser, AST, semantic analysis, and LLVM/Clang code generator).
- `std/` - Standard library definitions.
- `tools/` - Development tools such as the code formatter.
- `tests/` - High-level tests.

## Local Setup

To get started, make sure you have the following installed:
1. **Rust**: The latest stable version of Rust (via `rustup`).
2. **Clang**: Clang is required to link and compile the generated LLVM IR into executable binaries.

Clone the repository and build the compiler:
```bash
cargo build
```

## Running Tests

We have unit tests for the components and integration tests that verify compiler execution. Run them with:
```bash
cargo test
```

## Code Formatting

All code must be formatted using `rustfmt` before committing:
```bash
cargo fmt
```
