use std::collections::BTreeMap;

#[derive(Clone, Debug, PartialEq)]
pub enum JValue {
    Null,
    Bool(bool),
    Num(String),
    Str(String),
    Arr(Vec<JValue>),
    Obj(BTreeMap<String, JValue>),
}

pub fn parse(input: &str) -> Result<JValue, String> {
    let mut p = Parser::new(input);
    let v = p.parse_value()?;
    p.skip_ws();
    if p.peek().is_some() {
        return Err("trailing characters after JSON value".to_string());
    }
    Ok(v)
}

struct Parser<'a> {
    src: &'a [u8],
    pos: usize,
}

impl<'a> Parser<'a> {
    fn new(s: &'a str) -> Self {
        Self {
            src: s.as_bytes(),
            pos: 0,
        }
    }

    fn peek(&self) -> Option<u8> {
        self.src.get(self.pos).copied()
    }

    fn next(&mut self) -> Option<u8> {
        let c = self.peek()?;
        self.pos += 1;
        Some(c)
    }

    fn skip_ws(&mut self) {
        while let Some(c) = self.peek() {
            if matches!(c, b' ' | b'\n' | b'\r' | b'\t') {
                self.pos += 1;
            } else {
                break;
            }
        }
    }

    fn expect_byte(&mut self, b: u8) -> Result<(), String> {
        match self.next() {
            Some(c) if c == b => Ok(()),
            _ => Err(format!("expected byte '{}', at {}", b as char, self.pos)),
        }
    }

    fn parse_value(&mut self) -> Result<JValue, String> {
        self.skip_ws();
        match self.peek() {
            Some(b'"') => self.parse_string().map(JValue::Str),
            Some(b'{') => self.parse_object(),
            Some(b'[') => self.parse_array(),
            Some(b't') => {
                self.expect_literal("true")?;
                Ok(JValue::Bool(true))
            }
            Some(b'f') => {
                self.expect_literal("false")?;
                Ok(JValue::Bool(false))
            }
            Some(b'n') => {
                self.expect_literal("null")?;
                Ok(JValue::Null)
            }
            Some(b'-' | b'0'..=b'9') => self.parse_number().map(JValue::Num),
            _ => Err(format!("unexpected token at {}", self.pos)),
        }
    }

    fn expect_literal(&mut self, lit: &str) -> Result<(), String> {
        for b in lit.as_bytes() {
            if self.next() != Some(*b) {
                return Err(format!("invalid literal at {}", self.pos));
            }
        }
        Ok(())
    }

    fn parse_string(&mut self) -> Result<String, String> {
        self.expect_byte(b'"')?;
        let mut out = String::new();
        loop {
            let c = self.next().ok_or_else(|| "unterminated string".to_string())?;
            match c {
                b'"' => return Ok(out),
                b'\\' => {
                    let esc = self.next().ok_or_else(|| "bad escape".to_string())?;
                    match esc {
                        b'"' => out.push('"'),
                        b'\\' => out.push('\\'),
                        b'/' => out.push('/'),
                        b'b' => out.push('\u{0008}'),
                        b'f' => out.push('\u{000C}'),
                        b'n' => out.push('\n'),
                        b'r' => out.push('\r'),
                        b't' => out.push('\t'),
                        b'u' => {
                            let code = self.parse_hex4()?;
                            if let Some(ch) = char::from_u32(code) {
                                out.push(ch);
                            }
                        }
                        _ => return Err("unknown escape".to_string()),
                    }
                }
                x => out.push(x as char),
            }
        }
    }

    fn parse_hex4(&mut self) -> Result<u32, String> {
        let mut v = 0u32;
        for _ in 0..4 {
            let c = self.next().ok_or_else(|| "short unicode escape".to_string())?;
            v <<= 4;
            v += match c {
                b'0'..=b'9' => (c - b'0') as u32,
                b'a'..=b'f' => (c - b'a' + 10) as u32,
                b'A'..=b'F' => (c - b'A' + 10) as u32,
                _ => return Err("invalid unicode escape".to_string()),
            };
        }
        Ok(v)
    }

    fn parse_number(&mut self) -> Result<String, String> {
        let start = self.pos;
        if self.peek() == Some(b'-') {
            self.pos += 1;
        }
        if self.peek() == Some(b'0') {
            self.pos += 1;
        } else {
            self.take_digits();
        }
        if self.peek() == Some(b'.') {
            self.pos += 1;
            self.take_digits();
        }
        if matches!(self.peek(), Some(b'e' | b'E')) {
            self.pos += 1;
            if matches!(self.peek(), Some(b'+' | b'-')) {
                self.pos += 1;
            }
            self.take_digits();
        }
        std::str::from_utf8(&self.src[start..self.pos])
            .map(|s| s.to_string())
            .map_err(|_| "invalid number".to_string())
    }

    fn take_digits(&mut self) {
        while matches!(self.peek(), Some(b'0'..=b'9')) {
            self.pos += 1;
        }
    }

    fn parse_array(&mut self) -> Result<JValue, String> {
        self.expect_byte(b'[')?;
        self.skip_ws();
        let mut out = Vec::new();
        if self.peek() == Some(b']') {
            self.pos += 1;
            return Ok(JValue::Arr(out));
        }
        loop {
            let v = self.parse_value()?;
            out.push(v);
            self.skip_ws();
            match self.next() {
                Some(b',') => {
                    self.skip_ws();
                }
                Some(b']') => return Ok(JValue::Arr(out)),
                _ => return Err("unterminated array".to_string()),
            }
        }
    }

    fn parse_object(&mut self) -> Result<JValue, String> {
        self.expect_byte(b'{')?;
        self.skip_ws();
        let mut out = BTreeMap::new();
        if self.peek() == Some(b'}') {
            self.pos += 1;
            return Ok(JValue::Obj(out));
        }
        loop {
            self.skip_ws();
            let key = self.parse_string()?;
            self.skip_ws();
            self.expect_byte(b':')?;
            self.skip_ws();
            let val = self.parse_value()?;
            out.insert(key, val);
            self.skip_ws();
            match self.next() {
                Some(b',') => {
                    self.skip_ws();
                }
                Some(b'}') => return Ok(JValue::Obj(out)),
                _ => return Err("unterminated object".to_string()),
            }
        }
    }
}
