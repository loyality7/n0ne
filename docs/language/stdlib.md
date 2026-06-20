# Standard Library

## Global Built-ins

No import needed.

```
print(value)        # write to stdout — accepts string, int, float
print_err(value)    # write to stderr — accepts string, int, float

c_argc()            # number of CLI arguments (int)
c_argv(index)       # CLI argument at index (string)
```

---

## io

```
use io

line = io.read()    # read one line from stdin, returns string
```

---

## fs

```
use fs

result = fs.read("file.txt")              # result[string]
result = fs.write("file.txt", "content") # result[void]
exists = fs.exists("file.txt")           # bool
result = fs.delete("file.txt")           # result[void]
result = fs.mkdir("dir/path")            # result[void]
result = fs.list("dir/path")             # result[list[string]]
```

All `fs` functions except `exists` return `result[T]`. Always check `.is_ok` or `.is_err`.

---

## json

```
use json

# encode any value to a JSON string
json.encode("hello")        # → "\"hello\""
json.encode(42)             # → "42"
json.encode(true)           # → "true"
json.encode([1, 2, 3])      # → "[1,2,3]"
json.encode({"k": "v"})     # → "{\"k\":\"v\"}"

# decode a JSON object string → result[map[string, string]]
result = json.decode("{\"app\": \"demo\"}")
if result.is_ok
    cfg = result.unwrap()
    if cfg.has("app")
        print(cfg.get("app").unwrap())
```

---

## Built-in Methods

### string

```
s.len()              # int
s.upper()            # string
s.lower()            # string
s.trim()             # string
s.contains("x")      # bool
s.starts_with("x")   # bool
s.ends_with("x")     # bool
s.replace("a", "b")  # string
s.split(",")         # list[string]
s.slice(0, 3)        # string
s.to_int()           # option[int]
s.to_float()         # option[float]
```

### int / float

```
n.to_string()        # string
n.to_float()         # float   (int only)
n.to_int()           # int     (float only)
```

### list

```
items.len()          # int
items.push(val)      # void
items.pop()          # option[T]
items.first()        # option[T]
items.last()         # option[T]
items.contains(val)  # bool
```

### map

```
m.get("key")         # option[V]
m.set("key", val)    # void
m.has("key")         # bool
m.keys()             # list[string]
m.values()           # list[V]
m.delete("key")      # void
```
