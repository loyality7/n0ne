# Language Reference

## Types

| Type | Description | Example |
|---|---|---|
| `int` | whole number | `x = 42` |
| `float` | decimal | `pi = 3.14` |
| `string` | text | `name = "n0ne"` |
| `bool` | true/false | `ok = true` |
| `list[T]` | ordered collection | `items = [1, 2, 3]` |
| `map[K, V]` | key-value pairs | `data = {"a": "b"}` |
| `result[T]` | value or error | returned by fs, json |
| `option[T]` | value or nothing | returned by list.pop, map.get |

---

## Functions

```
fn add(a: int, b: int) -> int
    return a + b

fn greet(name: string) -> string
    return "hello " + name

# no return type = returns nothing
fn log(msg: string)
    print(msg)
```

---

## Entry Point

```
task main
    print("hello world")
```

---

## Control Flow

```
if x > 10
    print("big")
elif x > 5
    print("medium")
else
    print("small")
```

```
for item in [1, 2, 3]
    print(item.to_string())
```

---

## Error Handling

```
result = fs.read("config.txt")

if result.is_ok
    content = result.unwrap()
    print(content)

if result.is_err
    print_err(result.error)
```

Use `try` to propagate errors automatically:

```
content = try fs.read("config.txt")
```

---

## String Interpolation

```
name = "world"
print(f"hello {name}")
print(f"2 + 2 = {2 + 2}")
```

---

## Custom Types

```
type Point
    x: int
    y: int

p = make_point()
p.x = 10
p.y = 20
```

---

## Comments

```
# this is a comment
```

---

## Indentation

4 spaces. Tabs are not allowed.
