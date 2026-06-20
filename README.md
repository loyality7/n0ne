# n0ne

Statically-typed, indentation-sensitive language compiled directly to native binaries via LLVM.

## What is n0ne
n0ne is designed to be minimal. It removes noise like curly braces and semicolons, replacing them with strict indentation rules. It compiles directly to native machine code without virtual machines or interpreters, using LLVM as its backend and clang for linking.

## Install
Ensure you have `clang` installed on your system.

### Windows (PowerShell)
```powershell
irm https://raw.githubusercontent.com/loyality7/n0ne/main/scripts/install.ps1 | iex
```

### Linux
```bash
curl -fsSL https://raw.githubusercontent.com/loyality7/n0ne/main/scripts/install.sh | sh
```

### macOS
*Support coming soon.*

## Hello World

### Code (`hello.n0`)
```python
task main
    show("hello world")
```

### Output
```bash
$ n0ne run hello.n0
hello world
```

## Language Features

- **Indentation-Sensitive Blocks**: No brackets or semicolons. Tabs are forbidden. Indents must be 4 spaces.
  ```python
  if true
      show("nested block")
  ```
- **Type Inference**: Variables infer their types from values on declaration.
  ```python
  name = "n0ne"  # Type: string
  version = 1    # Type: int
  ```
- **Option and Result Types**: Robust error handling and null safety built in.
  ```python
  v = some("val")
  res = ok(42)
  ```
- **Native List and Map Literals**: Lists and maps initialized with a clean syntax.
  ```python
  nums = [1, 2, 3]
  data = {"x": "hello"}
  ```
- **F-Strings**: Clean, in-string expressions.
  ```python
  show(f"version: {version}")
  ```

## Examples

### 1. Command Line Interface Tool
```python
task main
    args_count = c_argc()
    if args_count < 2
        show("Usage: program <name>")
        return

    name = c_argv(1)
    show(f"Running command-line tool for: {name}")
```

### 2. File Reader
```python
use fs
use io

task main
    content = fs.read_to_string("data.txt")
    match content
        ok(text) =>
            io.show(text)
        err(msg) =>
            io.show_err(f"Failed to read file: {msg}")
```

### 3. Data Processor
```python
fn process(val: int) -> result[int]
    if val < 0
        return err("value cannot be negative")
    return ok(val * 2)

task main
    val = try process(10)
    match val
        ok(res) =>
            show(f"Processed: {res}")
        err(e) =>
            show(f"Error: {e}")
```

## Syntax Overview

### Variables
Declared by assign, types inferred or annotated.
```python
x = 42
```

### Functions
```python
fn double(n: int) -> int
    return n * 2
```

### Types
Struct-like definitions with attached methods.
```python
type User
    name: string
    age: int

fn (self User) greet()
    show(f"Hi, I'm {self.name}")
```

### Control Flow
```python
if score > 90
    show("Pass")
else
    show("Fail")
```

### Error Handling
Explicit option and result matching.
```python
val = try risky_fn()
if val.is_err
    show(val.error)
```

## Toolchain Commands

### `n0ne build`
Compile a source file into a native binary.
```bash
n0ne build hello.n0
```

### `n0ne run`
Compile and run the program immediately.
```bash
n0ne run hello.n0
```

### `n0ne fmt`
Format source code matching standard syntax rules.
```bash
n0ne fmt hello.n0
```

### `n0ne test`
Run all tests inside a source file.
```bash
n0ne test
```

## Stdlib
Standard modules embedded in the compiler.

- **`io`**: Text terminal display and console printing (`show`, `show_err`).
- **`fs`**: File system operations (`read_to_string`, `exists`).
- **`json`**: Basic serialization and parsing.
- **`http`**: HTTP requests and response retrieval.

## Project Status
**Current Version**: `v0.1.1` (Stably passed the full integration suite).

### What works
- AST Parser, Indentation-sensitive Lexer, Semantic analyzer.
- LLVM IR generation.
- Result/Option checking, map value propagation, f-strings, list/map literals.

### What is coming next
- Complete Standard Library Core (filesystem writes, socket networking).
- Package manager integration (`n0ne add`).
- Cross-platform support for macOS arm64.

## Contributing

### Running Tests
Requires a stable Rust toolchain and Clang.
```bash
cargo test
```

### Submitting Bugs
Please open an issue describing the code block, expected output, and actual LLVM IR generation output.

### Coding Style
All contributions should match the formatting output of:
```bash
n0ne fmt <file>
```

## License
MIT License. See `LICENSE` for details.
