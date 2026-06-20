pub(crate) struct Cursor<'a> {
    pub(crate) chars: std::str::Chars<'a>,
    pub(crate) line: usize,
    pub(crate) column: usize,
}

impl<'a> Cursor<'a> {
    pub(crate) fn new(input: &'a str) -> Self {
        Self {
            chars: input.chars(),
            line: 1,
            column: 1,
        }
    }

    pub(crate) fn peek(&self) -> Option<char> {
        self.chars.clone().next()
    }

    pub(crate) fn peek_next(&self) -> Option<char> {
        let mut cl = self.chars.clone();
        cl.next();
        cl.next()
    }

    pub(crate) fn peek_third(&self) -> Option<char> {
        let mut cl = self.chars.clone();
        cl.next();
        cl.next();
        cl.next()
    }

    pub(crate) fn bump(&mut self) -> Option<char> {
        if let Some(c) = self.chars.next() {
            if c == '\t' {
                panic!(
                    "Lexical error: Tab character is not allowed. (line {}, column {})",
                    self.line, self.column
                );
            }
            if c == '\n' {
                self.line += 1;
                self.column = 1;
            } else {
                self.column += 1;
            }
            Some(c)
        } else {
            None
        }
    }
}
