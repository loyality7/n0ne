# n0ne

Build software. Not complexity.

---

n0ne is a statically typed, compiled programming language. No braces. No semicolons. No runtime. Just clean code that compiles to fast native binaries.

---

## Why n0ne

Python feels good but runs slow and needs a runtime.
Go is efficient but verbose in places.
Rust is powerful but complex to learn.

n0ne sits in the middle. Readable as Python. Compiled as Go. No complexity tax.

---

## Install

### Windows

```powershell
irm https://raw.githubusercontent.com/loyality7/n0ne/main/scripts/install.ps1 | iex
```

### Linux

```bash
curl -fsSL https://raw.githubusercontent.com/loyality7/n0ne/main/scripts/install.sh | sh
```

### macOS

Coming soon.

---

## Hello World

```n0
task main
    show("hello world")
```

```bash
$ n0ne run hello.n0
hello world
```

---

## Language

### Variables

```n0
name    = "sarath"
age     = 21
score   = 99.5
active  = true
const MAX = 100
```

### Functions

```n0
fn add(a: int, b: int) -> int
    return a + b

task main
    show(add(3, 4).to_string())
```

### Types and Methods

```n0
type User
    name : string
    age  : int

fn (self User) greet()
    show(f"hi i am {self.name}")

task main
    u = User()
    u.name = "sarath"
    u.age = 21
    u.greet()
```

### If / Elif / Else

```n0
task main
    x = 42
    if x > 100
        show("big")
    elif x > 10
        show("medium")
    else
        show("small")
```

### Match

```n0
task main
    status = "active"
    match status
        "active"  -> show("running")
        "stopped" -> show("halted")
        _         -> show("unknown")
```

### While Loop

```n0
task main
    x = 5
    while x > 0
        show(x.to_string())
        x = x - 1
```

### For Loop

```n0
task main
    names = ["sarath", "john", "jane"]
    for name in names
        show(f"hello {name}")
```

### Break and Continue

```n0
task main
    for i in [1, 2, 3, 4, 5]
        if i == 3
            continue
        if i == 5
            break
        show(i.to_string())
```

### F-Strings

```n0
task main
    name = "sarath"
    age  = 21
    show(f"name: {name}, age: {age}")
```

### Multiline Strings

```n0
task main
    text = """
hello
world
"""
    show(text)
```

### Lists

```n0
task main
    items = [1, 2, 3]
    items.push(4)
    show(items.len().to_string())
    for item in items
        show(item.to_string())
```

### Maps

```n0
task main
    data = {"name": "sarath", "city": "bangalore"}
    v = data.get("name")
    if v.is_some
        show(v.unwrap())
```

### Result and Error Handling

```n0
fn divide(a: int, b: int) -> result[int]
    if b == 0
        return err("cannot divide by zero")
    return ok(a / b)

task main
    r = divide(10, 2)
    if r.is_ok
        show(r.unwrap().to_string())

    r2 = divide(10, 0)
    if r2.is_err
        show(r2.error)
```

### Try Keyword

```n0
use fs
use io

fn load(path: string) -> result[string]
    data = try fs.read(path)
    return ok(data)

task main
    result = load("config.txt")
    if result.is_err
        show(f"error: {result.error}")
    else
        show(result.unwrap())
```

### Option Type

```n0
fn find(id: int) -> option[string]
    users = {1: "sarath", 2: "john"}
    return users.get(id)

task main
    u = find(1)
    if u.is_some
        show(u.unwrap())
    u2 = find(99)
    if u2.is_none
        show("not found")
```

---

## Real Examples

### CLI Tool

```n0
use io

fn main(args: list[string])
    if args.len() == 0
        show("usage: tool <name>")
        return
    show(f"hello {args[0]}")
```

### File Reader

```n0
use fs
use io

task main
    data = try fs.read("notes.txt")
    lines = data.split("\n")
    for line in lines
        if line.contains("TODO")
            show(f"found: {line}")
```

### Data Processor

```n0
fn process(val: int) -> result[int]
    if val < 0
        return err("negative not allowed")
    return ok(val * 2)

task main
    nums = [10, -1, 5, -3, 8]
    for n in nums
        r = process(n)
        if r.is_ok
            show(r.unwrap().to_string())
        else
            show(f"skip: {r.error}")
```

### Type System Example

```n0
type Product
    name  : string
    price : float
    stock : int

fn (self Product) is_available() -> bool
    return self.stock > 0

fn (self Product) display()
    show(f"{self.name} - ${self.price}")

task main
    p = Product()
    p.name  = "keyboard"
    p.price = 99.99
    p.stock = 5

    if p.is_available()
        p.display()
    else
        show("out of stock")
```

---

## Toolchain

| Command | What it does |
|---|---|
| `n0ne build file.n0` | compile to native binary |
| `n0ne run file.n0` | compile and run immediately |
| `n0ne fmt file.n0` | format source file |
| `n0ne test` | run all tests |

---

## Standard Library

| Module | Status | What it does |
|---|---|---|
| `io` | ✓ done | show, show_err, read |
| `fs` | ✓ done | read, write, exists, delete, mkdir, list |
| `json` | coming | encode, decode |
| `http` | coming | get, post, basic server |

---

## How It Works

```
your .n0 file
     |
     v
   lexer          tokenize. indent/dedent stack.
     |
     v
   parser         recursive descent. builds AST.
     |
     v
   sema           type check. scope check. error check.
     |
     v
 llvm codegen     emit LLVM IR text
     |
     v
   clang          compile IR + link C runtime
     |
     v
 native binary    .exe on Windows. ELF on Linux.
```

No VM. No interpreter. No garbage collector. Direct to machine code.

---

## Project Status

| Version | Status | What |
|---|---|---|
| v0.1 | ✓ shipped | core language. lexer parser sema codegen. |
| v0.1.1 | ✓ shipped | while. match. const. break. continue. multiline strings. 87 tests. |
| v0.2 | in progress | stdlib complete. json. http. |
| v0.3 | planned | package manager. n0ne add. |
| v1.0 | planned | stable. LSP. cross compile. |

---

## Contributing

Clone and build:

```bash
git clone https://github.com/loyality7/n0ne
cd n0ne
cargo build
```

Run tests:

```bash
cargo test
```

Format all n0ne examples:

```bash
n0ne fmt examples/hello/main.n0
```

Submit bugs with:
- the n0ne source code that fails
- expected output
- actual output
- your OS and n0ne version

---

## Requirements

- clang installed on system
- Windows x64 or Linux x64
- Rust stable (to build from source)

---

## License

MIT. See LICENSE file.

---

n0ne. Build software. Not complexity.
