use std::collections::HashMap;
use std::io::{self, BufRead, Write};
use std::path::PathBuf;
use serde_json::Value;

pub fn start_lsp_server() -> io::Result<()> {
    let stdin = io::stdin();
    let stdout = io::stdout();
    let mut reader = io::BufReader::new(stdin.lock());
    let mut writer = io::BufWriter::new(stdout.lock());

    let mut server = LspServer::new();

    loop {
        if let Some(msg_str) = read_message(&mut reader)? {
            if let Ok(req) = serde_json::from_str::<Value>(&msg_str) {
                if let Some(resp) = server.handle_message(req) {
                    write_message(&mut writer, &resp)?;
                }
            }
        } else {
            break; // stdin closed
        }
    }
    Ok(())
}

fn read_message<R: BufRead>(reader: &mut R) -> io::Result<Option<String>> {
    let mut header = String::new();
    loop {
        let mut line = String::new();
        let n = reader.read_line(&mut line)?;
        if n == 0 {
            return Ok(None);
        }
        if line == "\r\n" || line == "\n" {
            break;
        }
        header.push_str(&line);
    }

    let mut content_length = 0;
    for line in header.lines() {
        if line.to_lowercase().starts_with("content-length:") {
            let parts: Vec<&str> = line.split(':').collect();
            if parts.len() == 2 {
                content_length = parts[1].trim().parse::<usize>().unwrap_or(0);
            }
        }
    }

    if content_length == 0 {
        return Ok(None);
    }

    let mut buf = vec![0u8; content_length];
    reader.read_exact(&mut buf)?;
    Ok(Some(String::from_utf8_lossy(&buf).into_owned()))
}

fn write_message<W: Write>(writer: &mut W, value: &Value) -> io::Result<()> {
    let payload = serde_json::to_string(value)?;
    let msg = format!("Content-Length: {}\r\n\r\n{}", payload.len(), payload);
    writer.write_all(msg.as_bytes())?;
    writer.flush()?;
    Ok(())
}

struct LspServer {
    documents: HashMap<String, String>,
}

impl LspServer {
    fn new() -> Self {
        Self {
            documents: HashMap::new(),
        }
    }

    fn handle_message(&mut self, msg: Value) -> Option<Value> {
        let method = msg.get("method")?.as_str()?;
        let id = msg.get("id").cloned();

        match method {
            "initialize" => {
                let result = serde_json::json!({
                    "capabilities": {
                        "textDocumentSync": 1, // Full
                        "completionProvider": {
                            "triggerCharacters": [".", "|>"]
                        },
                        "hoverProvider": true,
                        "definitionProvider": true,
                        "referencesProvider": true
                    }
                });
                Some(serde_json::json!({
                    "jsonrpc": "2.0",
                    "id": id,
                    "result": result
                }))
            }
            "initialized" => None,
            "textDocument/didOpen" => {
                let params = msg.get("params")?;
                let doc = params.get("textDocument")?;
                let uri = doc.get("uri")?.as_str()?.to_string();
                let text = doc.get("text")?.as_str()?.to_string();
                self.documents.insert(uri.clone(), text.clone());
                
                // Publish diagnostics
                let diagnostics = self.compute_diagnostics(&uri, &text);
                let notification = serde_json::json!({
                    "jsonrpc": "2.0",
                    "method": "textDocument/publishDiagnostics",
                    "params": {
                        "uri": uri,
                        "diagnostics": diagnostics
                    }
                });
                Some(notification)
            }
            "textDocument/didChange" => {
                let params = msg.get("params")?;
                let doc = params.get("textDocument")?;
                let uri = doc.get("uri")?.as_str()?.to_string();
                let changes = params.get("contentChanges")?.as_array()?;
                if let Some(change) = changes.first() {
                    let text = change.get("text")?.as_str()?.to_string();
                    self.documents.insert(uri.clone(), text.clone());
                    
                    let diagnostics = self.compute_diagnostics(&uri, &text);
                    let notification = serde_json::json!({
                        "jsonrpc": "2.0",
                        "method": "textDocument/publishDiagnostics",
                        "params": {
                            "uri": uri,
                            "diagnostics": diagnostics
                        }
                    });
                    return Some(notification);
                }
                None
            }
            "textDocument/completion" => {
                let params = msg.get("params")?;
                let doc = params.get("textDocument")?;
                let uri = doc.get("uri")?.as_str()?;
                let pos = params.get("position")?;
                let line = pos.get("line")?.as_u64()? as usize;
                let col = pos.get("character")?.as_u64()? as usize;

                let completions = self.compute_completions(uri, line, col);
                Some(serde_json::json!({
                    "jsonrpc": "2.0",
                    "id": id,
                    "result": completions
                }))
            }
            "textDocument/hover" => {
                let params = msg.get("params")?;
                let doc = params.get("textDocument")?;
                let uri = doc.get("uri")?.as_str()?;
                let pos = params.get("position")?;
                let line = pos.get("line")?.as_u64()? as usize;
                let col = pos.get("character")?.as_u64()? as usize;

                let hover = self.compute_hover(uri, line, col);
                Some(serde_json::json!({
                    "jsonrpc": "2.0",
                    "id": id,
                    "result": hover
                }))
            }
            "textDocument/definition" => {
                let params = msg.get("params")?;
                let doc = params.get("textDocument")?;
                let uri = doc.get("uri")?.as_str()?;
                let pos = params.get("position")?;
                let line = pos.get("line")?.as_u64()? as usize;
                let col = pos.get("character")?.as_u64()? as usize;

                let definition = self.compute_definition(uri, line, col);
                Some(serde_json::json!({
                    "jsonrpc": "2.0",
                    "id": id,
                    "result": definition
                }))
            }
            "textDocument/references" => {
                let params = msg.get("params")?;
                let doc = params.get("textDocument")?;
                let uri = doc.get("uri")?.as_str()?;
                let pos = params.get("position")?;
                let line = pos.get("line")?.as_u64()? as usize;
                let col = pos.get("character")?.as_u64()? as usize;

                let refs = self.compute_references(uri, line, col);
                Some(serde_json::json!({
                    "jsonrpc": "2.0",
                    "id": id,
                    "result": refs
                }))
            }
            "shutdown" => {
                Some(serde_json::json!({
                    "jsonrpc": "2.0",
                    "id": id,
                    "result": serde_json::Value::Null
                }))
            }
            "exit" => {
                std::process::exit(0);
            }
            _ => None
        }
    }

    fn compute_diagnostics(&self, uri: &str, text: &str) -> Vec<Value> {
        let path = if let Some(p) = uri_to_path(uri) {
            p
        } else {
            PathBuf::from("file.n0")
        };

        let mut diagnostics = Vec::new();

        let ast_res = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let tokens = lexer::Lexer::tokenize(text);
            let mut parser = parser::Parser::new(tokens);
            parser.parse()
        }));

        match ast_res {
            Ok(ast) => {
                let mut checker = sema::TypeChecker::new();
                checker.current_file = Some(path.clone());
                checker.check_program(&ast);

                for mut warn in checker.warnings {
                    crate::resolve_location(text, &mut warn);
                    diagnostics.push(make_diagnostic(2, &warn));
                }

                for mut err in checker.errors {
                    crate::resolve_location(text, &mut err);
                    diagnostics.push(make_diagnostic(1, &err));
                }
            }
            Err(err) => {
                let msg = if let Some(s) = err.downcast_ref::<&str>() {
                    *s
                } else if let Some(s) = err.downcast_ref::<String>() {
                    s.as_str()
                } else {
                    "Unknown syntax error"
                };

                if let Some((code, line, col, clean_msg, hint)) = crate::parse_syntax_panic(msg) {
                    let l = if line > 0 { line - 1 } else { 0 };
                    let c = if col > 0 { col - 1 } else { 0 };
                    diagnostics.push(serde_json::json!({
                        "range": {
                            "start": { "line": l, "character": c },
                            "end": { "line": l, "character": c + 1 }
                        },
                        "severity": 1,
                        "code": code,
                        "message": format!("{}\nHint: {}", clean_msg, hint),
                        "source": "n0ne"
                    }));
                } else {
                    diagnostics.push(serde_json::json!({
                        "range": {
                            "start": { "line": 0, "character": 0 },
                            "end": { "line": 0, "character": 1 }
                        },
                        "severity": 1,
                        "message": msg,
                        "source": "n0ne"
                    }));
                }
            }
        }

        diagnostics
    }

    fn compute_completions(&self, uri: &str, line: usize, col: usize) -> Value {
        let text = match self.documents.get(uri) {
            Some(t) => t,
            None => return serde_json::json!([]),
        };

        let lines: Vec<&str> = text.lines().collect();
        if line >= lines.len() {
            return serde_json::json!([]);
        }
        let current_line = lines[line];

        let mut is_dot = false;
        let mut prefix = "";
        
        let chars: Vec<char> = current_line.chars().collect();
        if col > 0 && col <= chars.len() {
            if chars[col - 1] == '.' {
                is_dot = true;
                let mut start = col - 1;
                while start > 0 && (chars[start - 1].is_alphanumeric() || chars[start - 1] == '_') {
                    start -= 1;
                }
                if start < col - 1 {
                    prefix = &current_line[start..col - 1];
                }
            }
        }

        let mut items = Vec::new();

        if is_dot {
            match prefix {
                "math" => {
                    for m in &["abs", "sqrt", "floor", "ceil", "round", "min", "max", "clamp", "random", "random_int", "PI", "E"] {
                        items.push(make_completion_item(m, 3));
                    }
                }
                "json" => {
                    for m in &["encode", "decode"] {
                        items.push(make_completion_item(m, 3));
                    }
                }
                "env" => {
                    for m in &["get", "set", "all"] {
                        items.push(make_completion_item(m, 3));
                    }
                }
                "process" => {
                    for m in &["run", "exit", "args"] {
                        items.push(make_completion_item(m, 3));
                    }
                }
                "string" => {
                    for m in &["pad_left", "pad_right", "repeat", "to_bytes", "from_bytes", "concat", "len", "lower", "upper", "trim", "replace", "split", "starts_with", "ends_with", "contains"] {
                        items.push(make_completion_item(m, 3));
                    }
                }
                "time" => {
                    for m in &["now", "sleep", "format"] {
                        items.push(make_completion_item(m, 3));
                    }
                }
                _ => {}
            }
        } else {
            for kw in &[
                "fn", "task", "type", "enum", "const", "if", "else", "elif", "while", "for", "break",
                "continue", "return", "defer", "guard", "print", "show", "true", "false",
                "math", "json", "env", "process", "time", "string"
            ] {
                items.push(make_completion_item(kw, 14));
            }

            let mut defined = std::collections::HashSet::new();
            for l in lines {
                let trimmed = l.trim();
                if trimmed.starts_with("fn ") {
                    if let Some(name) = trimmed["fn ".len()..].split('(').next() {
                        defined.insert(name.trim().to_string());
                    }
                } else if trimmed.starts_with("task ") {
                    if let Some(name) = trimmed["task ".len()..].split_whitespace().next() {
                        defined.insert(name.trim().to_string());
                    }
                } else if trimmed.starts_with("const ") {
                    if let Some(name) = trimmed["const ".len()..].split('=').next() {
                        defined.insert(name.trim().to_string());
                    }
                } else if trimmed.starts_with("type ") {
                    if let Some(name) = trimmed["type ".len()..].split('=').next() {
                        defined.insert(name.trim().to_string());
                    }
                } else if trimmed.starts_with("enum ") {
                    if let Some(name) = trimmed["enum ".len()..].split_whitespace().next() {
                        defined.insert(name.trim().to_string());
                    }
                }
            }

            for name in defined {
                items.push(make_completion_item(&name, 6));
            }
        }

        serde_json::json!(items)
    }

    fn compute_hover(&self, uri: &str, line: usize, col: usize) -> Value {
        let text = match self.documents.get(uri) {
            Some(t) => t,
            None => return serde_json::Value::Null,
        };

        let word = match get_word_at(text, line, col) {
            Some(w) => w,
            None => return serde_json::Value::Null,
        };

        let lines: Vec<&str> = text.lines().collect();
        let line_content = lines[line];
        let full_word = if let Some(idx) = line_content.find(&word) {
            if idx > 0 && &line_content[idx - 1..idx] == "." {
                let mut start = idx - 1;
                while start > 0 && line_content.chars().nth(start - 1).map_or(false, |c| c.is_alphanumeric()) {
                    start -= 1;
                }
                format!("{}.{}", &line_content[start..idx - 1], word)
            } else {
                word.clone()
            }
        } else {
            word.clone()
        };

        let stdlib_docs = get_stdlib_hover_doc(&full_word);
        if let Some(doc) = stdlib_docs {
            return serde_json::json!({
                "contents": {
                    "kind": "markdown",
                    "value": doc
                }
            });
        }

        if let Some((def_line, def_col, desc)) = self.find_definition_in_text(text, &word) {
            return serde_json::json!({
                "contents": {
                    "kind": "markdown",
                    "value": format!("```n0ne\n{}\n```\nDeclared at line {}, column {}", desc, def_line + 1, def_col + 1)
                }
            });
        }

        serde_json::Value::Null
    }

    fn find_definition_in_text(&self, text: &str, word: &str) -> Option<(usize, usize, String)> {
        let lines: Vec<&str> = text.lines().collect();

        for (i, line) in lines.iter().enumerate() {
            let trimmed = line.trim();
            if trimmed.starts_with(&format!("fn {}", word))
                || trimmed.starts_with(&format!("task {}", word))
                || trimmed.starts_with(&format!("type {}", word))
                || trimmed.starts_with(&format!("enum {}", word))
                || trimmed.starts_with(&format!("const {}", word))
            {
                if let Some(col) = line.find(word) {
                    return Some((i, col, line.trim().to_string()));
                }
            }
        }

        for (i, line) in lines.iter().enumerate() {
            if let Some(col) = crate::find_word(line, word) {
                let suffix = line[col + word.len()..].trim_start();
                if suffix.starts_with('=') || suffix.starts_with(':') {
                    return Some((i, col, line.trim().to_string()));
                }
            }
        }

        None
    }

    fn compute_definition(&self, uri: &str, line: usize, col: usize) -> Value {
        let text = match self.documents.get(uri) {
            Some(t) => t,
            None => return serde_json::Value::Null,
        };

        let word = match get_word_at(text, line, col) {
            Some(w) => w,
            None => return serde_json::Value::Null,
        };

        if let Some((def_line, def_col, _)) = self.find_definition_in_text(text, &word) {
            return serde_json::json!({
                "uri": uri,
                "range": {
                    "start": { "line": def_line, "character": def_col },
                    "end": { "line": def_line, "character": def_col + word.len() }
                }
            });
        }

        serde_json::Value::Null
    }

    fn compute_references(&self, uri: &str, line: usize, col: usize) -> Value {
        let text = match self.documents.get(uri) {
            Some(t) => t,
            None => return serde_json::json!([]),
        };

        let word = match get_word_at(text, line, col) {
            Some(w) => w,
            None => return serde_json::json!([]),
        };

        let mut refs = Vec::new();
        let lines: Vec<&str> = text.lines().collect();
        for (i, line_content) in lines.iter().enumerate() {
            let mut start = 0;
            while let Some(idx) = crate::find_word(&line_content[start..], &word) {
                let abs_idx = start + idx;
                refs.push(serde_json::json!({
                    "uri": uri,
                    "range": {
                        "start": { "line": i, "character": abs_idx },
                        "end": { "line": i, "character": abs_idx + word.len() }
                    }
                }));
                start = abs_idx + word.len();
            }
        }

        serde_json::json!(refs)
    }
}

fn make_diagnostic(severity: i32, err: &sema::SemanticError) -> Value {
    let line = if err.line > 0 { err.line - 1 } else { 0 };
    let col = if err.column > 0 { err.column - 1 } else { 0 };
    let hint_str = if err.hint.is_empty() {
        "".to_string()
    } else {
        format!("\nHint: {}", err.hint)
    };

    serde_json::json!({
        "range": {
            "start": { "line": line, "character": col },
            "end": { "line": line, "character": col + 1 }
        },
        "severity": severity,
        "code": err.code,
        "message": format!("{}{}", err.message, hint_str),
        "source": "n0ne"
    })
}

fn make_completion_item(label: &str, kind: i32) -> Value {
    serde_json::json!({
        "label": label,
        "kind": kind
    })
}

fn get_word_at(source: &str, line: usize, character: usize) -> Option<String> {
    let lines: Vec<&str> = source.lines().collect();
    if line >= lines.len() {
        return None;
    }
    let line_content = lines[line];
    let chars: Vec<char> = line_content.chars().collect();
    if character >= chars.len() {
        return None;
    }
    
    let mut start = character;
    while start > 0 && (chars[start - 1].is_alphanumeric() || chars[start - 1] == '_') {
        start -= 1;
    }
    
    let mut end = character;
    while end < chars.len() && (chars[end].is_alphanumeric() || chars[end] == '_') {
        end += 1;
    }
    
    if start == end {
        return None;
    }
    
    let word: String = chars[start..end].iter().collect();
    Some(word)
}

fn uri_to_path(uri: &str) -> Option<PathBuf> {
    let decoded = decode_uri(uri);
    let path_str = decoded.strip_prefix("file://")?;
    let path_str = if cfg!(target_os = "windows") && (path_str.starts_with('/') || path_str.starts_with('\\')) {
        &path_str[1..]
    } else {
        path_str
    };
    Some(PathBuf::from(path_str))
}

fn decode_uri(uri: &str) -> String {
    let mut res = String::new();
    let mut chars = uri.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '%' {
            let mut hex = String::new();
            if let Some(h1) = chars.next() { hex.push(h1); }
            if let Some(h2) = chars.next() { hex.push(h2); }
            if let Ok(val) = u8::from_str_radix(&hex, 16) {
                res.push(val as char);
            } else {
                res.push('%');
                res.push_str(&hex);
            }
        } else {
            res.push(c);
        }
    }
    res
}

fn get_stdlib_hover_doc(name: &str) -> Option<String> {
    let doc = match name {
        "math" => "Math standard library module containing mathematical constants and functions.",
        "math.abs" => "```n0ne\nmath.abs(n: int|float) -> int|float\n```\nReturns the absolute value of `n`.",
        "math.sqrt" => "```n0ne\nmath.sqrt(f: float) -> float\n```\nReturns the square root of `f`.",
        "math.floor" => "```n0ne\nmath.floor(f: float) -> float\n```\nReturns the largest integer less than or equal to `f`.",
        "math.ceil" => "```n0ne\nmath.ceil(f: float) -> float\n```\nReturns the smallest integer greater than or equal to `f`.",
        "math.round" => "```n0ne\nmath.round(f: float) -> float\n```\nReturns the nearest integer to `f`.",
        "math.min" => "```n0ne\nmath.min(a: int|float, b: int|float) -> int|float\n```\nReturns the minimum of `a` and `b`.",
        "math.max" => "```n0ne\nmath.max(a: int|float, b: int|float) -> int|float\n```\nReturns the maximum of `a` and `b`.",
        "math.clamp" => "```n0ne\nmath.clamp(val: float, low: float, high: float) -> float\n```\nClamps `val` to the range `[low, high]`.",
        "math.PI" => "```n0ne\nconst math.PI: float = 3.141592653589793\n```",
        "math.E" => "```n0ne\nconst math.E: float = 2.718281828459045\n```",
        "math.random" => "```n0ne\nmath.random() -> float\n```\nReturns a random float in range `[0.0, 1.0)`.",
        "math.random_int" => "```n0ne\nmath.random_int(min: int, max: int) -> int\n```\nReturns a random integer between `min` and `max` (inclusive).",
        "json" => "JSON standard library module for serialization and deserialization.",
        "json.encode" => "```n0ne\njson.encode(val: any) -> string\n```\nSerializes `val` to its JSON string representation.",
        "json.decode" => "```n0ne\njson.decode(s: string) -> result[map]\n```\nDeserializes `s` from a JSON string to a result map.",
        "env" => "Environment variables standard library module.",
        "env.get" => "```n0ne\nenv.get(key: string) -> option[string]\n```\nGets the value of environment variable `key`.",
        "env.set" => "```n0ne\nenv.set(key: string, val: string)\n```\nSets the environment variable `key` to `val`.",
        "env.all" => "```n0ne\nenv.all() -> map[string, string]\n```\nReturns a map of all environment variables.",
        "process" => "Process management standard library module.",
        "process.run" => "```n0ne\nprocess.run(cmd: string) -> result[string]\n```\nRuns a shell command and returns its standard output.",
        "process.exit" => "```n0ne\nprocess.exit(code: int)\n```\nExits the current process with exit code `code`.",
        "process.args" => "```n0ne\nprocess.args() -> list[string]\n```\nReturns a list of command line arguments passed to the program.",
        "string" => "String utility standard library functions.",
        "string.pad_left" => "```n0ne\nstring.pad_left(s: string, n: int) -> string\n```\nPads string `s` on the left with spaces to reach length `n`.",
        "string.pad_right" => "```n0ne\nstring.pad_right(s: string, n: int) -> string\n```\nPads string `s` on the right with spaces to reach length `n`.",
        "string.repeat" => "```n0ne\nstring.repeat(s: string, n: int) -> string\n```\nRepeats string `s`, `n` times.",
        "string.to_bytes" => "```n0ne\nstring.to_bytes(s: string) -> list[int]\n```\nConverts string `s` to a list of byte integers.",
        "string.from_bytes" => "```n0ne\nstring.from_bytes(bytes: list[int]) -> string\n```\nCreates a string from a list of byte integers.",
        "time" => "Time standard library module.",
        "time.now" => "```n0ne\ntime.now() -> int\n```\nReturns the current Unix epoch timestamp in seconds.",
        "time.sleep" => "```n0ne\ntime.sleep(ms: int)\n```\nPauses thread execution for `ms` milliseconds.",
        "time.format" => "```n0ne\ntime.format(ts: int, fmt: string) -> string\n```\nFormats timestamp `ts` with format `fmt`.",
        _ => return None,
    };
    Some(doc.to_string())
}
