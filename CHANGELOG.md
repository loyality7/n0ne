# Changelog

## [0.1.0] - 2026-06-20

First public release.

### Language

- `task main` as the program entry point
- `fn` for function definitions with typed arguments and return types
- `type` for custom struct-like types with named fields
- `if` / `else` / `elif` control flow
- `for x in list` loops
- `return` statement
- `use` for importing stdlib modules (`io`, `fs`, `json`)
- String interpolation with f-strings: `f"hello {name}"`
- Comments with `#`

### Built-in Output

- `print(value)` — write to stdout
- `print_err(value)` — write to stderr
- Both accept `string`, `int`, and `float`

### Error Handling

- `result[T]` and `option[T]` types
- `.is_ok` / `.is_err` field access
- `.unwrap()` to get the value
- `.error` to get the error string
- `try expr` to propagate errors early

### Standard Library

- **io** — `io.read()` to read a line from stdin
- **fs** — `fs.read()`, `fs.write()`, `fs.exists()`, `fs.delete()`, `fs.mkdir()`, `fs.list()`
- **json** — `json.encode(value)`, `json.decode(string)`

### Built-in Methods

- **string**: `.len()`, `.upper()`, `.lower()`, `.trim()`, `.contains()`, `.starts_with()`, `.ends_with()`, `.replace()`, `.split()`, `.slice()`, `.to_int()`, `.to_float()`
- **int / float**: `.to_string()`, `.to_int()`, `.to_float()`
- **list**: `.len()`, `.push()`, `.pop()`, `.first()`, `.last()`, `.contains()`
- **map**: `.get()`, `.set()`, `.has()`, `.keys()`, `.values()`, `.delete()`

### Tooling

- `n0ne build <file>` — compile to native executable
- `n0ne run <file>` — compile and run immediately
- `n0ne fmt <file>` — format source file
- One-line installer for Linux/macOS and Windows
- Clang auto-detected from system PATH — no manual setup required
- CI/CD pipeline with automated GitHub Releases for Linux and Windows
