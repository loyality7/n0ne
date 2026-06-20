# Changelog

All notable changes to the `n0ne` programming language compiler will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.0] - 2026-06-20

### Added
- **Primitive Type Methods**:
  - **String Methods**: `.len()`, `.contains()`, `.starts_with()`, `.ends_with()`, `.upper()`, `.lower()`, `.trim()`, `.split()`, `.replace()`, `.slice()`, `.to_int()`, and `.to_float()`.
  - **List Methods**: `.len()`, `.push()`, `.pop()`, `.contains()`, `.first()`, and `.last()`.
  - **Map Methods**: `.get()`, `.set()`, `.has()`, `.keys()`, `.values()`, and `.delete()`.
  - **Numeric Methods**: `.to_string()`, `.to_int()`, and `.to_float()`.
- **Option Type Layout**:
  - Implemented field-access layout alignment for `Option[T]` fields (`is_some` at `+8`, `is_none` at `+16`, and `value` at `+24`).
- **Collection Literals**:
  - Syntax, parser, semantic analysis, and C-runtime codegen support for List literals (e.g., `items = [1, 2, 3]`) and Map literals (e.g., `data = {"k": "v"}`).
- **String Interpolation (F-Strings)**:
  - Added support for formatting variables and expressions inside string templates using the syntax `f"hello {name}"`.
- **Integration Test Suite**:
  - Added full end-to-end integration tests compiler pipeline test coverage (`test_string_methods`, `test_list_methods`, `test_map_methods`, `test_numeric_methods`).

### Changed
- **Compiler Codebase Modularization**:
  - Split the original single 1,100+ line LLVM generator file (`compiler/codegen/llvm/src/lib.rs`) into distinct, single-responsibility modules:
    - `lib.rs`: entry point & declarations.
    - `emitter.rs`: low-level instruction emitter & offset helpers.
    - `expr.rs`: expression evaluation.
    - `stmt.rs`: statement & control-flow generation.
    - `types.rs`: type resolution and mappings.
    - `runtime.rs`: compiler-embedded C runtime.
    - `linker.rs`: Clang binary compilation driver.
    - `builtins.rs`: compiler built-in functions.
- **Improved C Runtime**:
  - Fixed safety issues inside `n0_map_delete` by avoiding freeing read-only static string constants.
  - Implemented dynamically resizing buffers for list and map collections.
