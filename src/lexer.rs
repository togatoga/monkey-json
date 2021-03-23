#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Symbol {
    LeftBrace,    // {　JSON object 開始文字
    RightBrace,   // }　JSON object 終了文字
    LeftBracket,  // [　JSON array  開始文字
    RightBracket, // ]　JSON array  終了文字
    Comma,        // ,　JSON value  区切り文字
    Colon,        // :　"key":value 区切り文字
}

#[derive(Debug, PartialEq, Clone)]
pub enum Token {
    String(String), // 文字列
    Number(f64),    // 数値
    Bool(bool),     // 真偽値
    Symbol(Symbol), // JSONの構文に必要な記号
    Null,           // Null
}

pub struct Lexer {
    chars: Vec<char>,
    index: usize,
}

#[derive(Debug)]
pub struct LexerError {
    pub msg: String,
}

impl LexerError {
    fn new(msg: &str) -> LexerError {
        LexerError {
            msg: msg.to_string(),
        }
    }
}

impl Lexer {
    pub fn new(input: &str) -> Lexer {
        Lexer {
            chars: input.chars().collect(),
            index: 0,
        }
    }
    fn advance(&mut self, x: usize) {
        self.index += x;
    }

    fn peek(&self) -> Option<char> {
        if self.index < self.chars.len() {
            Some(self.chars[self.index])
        } else {
            None
        }
    }

    fn skip_whitespace(&mut self) {
        while let Some(c) = self.peek() {
            if c.is_whitespace() || c == '\n' {
                self.advance(1);
            } else {
                break;
            }
        }
    }

    pub fn lex(&mut self) -> Result<Vec<Token>, LexerError> {
        let mut tokens = vec![];
        loop {
            self.skip_whitespace();
            if let Some(c) = self.peek() {
                match c {
                    '{' | '}' | '[' | ']' | ',' | ':' => {
                        let symbol = match c {
                            '{' => Symbol::LeftBrace,
                            '}' => Symbol::RightBrace,
                            '[' => Symbol::LeftBracket,
                            ']' => Symbol::RightBracket,
                            ',' => Symbol::Comma,
                            ':' => Symbol::Colon,
                            _ => {
                                return Err(LexerError::new(&format!(
                                    "Lexer Error: Unexpected char: {}",
                                    c
                                )));
                            }
                        };

                        tokens.push(Token::Symbol(symbol));
                        self.advance(1);
                    }
                    _ => {
                        // greedy match
                        if let Some(token) = self.parse_string() {
                            tokens.push(token);
                            continue;
                        }

                        if let Some(token) = self.parse_number() {
                            tokens.push(token);
                            continue;
                        }

                        if let Some(token) = self.parse_bool() {
                            tokens.push(token);
                            continue;
                        }

                        if let Some(token) = self.parse_null() {
                            tokens.push(token);
                            continue;
                        }
                        return Err(LexerError::new(
                            "Lexer Error: Failed all parsings (string, number, bool, null)",
                        ));
                    }
                }
            } else {
                break;
            }
        }

        Ok(tokens)
    }

    fn parse_bool(&mut self) -> Option<Token> {
        let b: String = self
            .chars
            .iter()
            .skip(self.index)
            .take_while(|x| matches!(x, 't' | 'r' | 'u' | 'e' | 'f' | 'a' | 'l' | 's'))
            .take(5)
            .collect();
        match b.as_str() {
            "true" => {
                self.advance(4);
                Some(Token::Bool(true))
            }
            "false" => {
                self.advance(5);
                Some(Token::Bool(false))
            }
            _ => None,
        }
    }
    fn parse_null(&mut self) -> Option<Token> {
        let n: String = self
            .chars
            .iter()
            .skip(self.index)
            .take_while(|x| matches!(x, 'n' | 'u' | 'l'))
            .take(4)
            .collect();
        match n.as_str() {
            "null" => {
                self.advance(4);
                Some(Token::Null)
            }
            _ => None,
        }
    }
    fn push_utf16(result: &mut String, utf16: &mut Vec<u16>) {
        if let Ok(utf16_str) = String::from_utf16(&utf16) {
            result.push_str(&utf16_str);
            utf16.clear();
        }
    }
    fn parse_string(&mut self) -> Option<Token> {
        if self.peek()? != '"' {
            return None;
        }
        let mut utf16 = vec![];
        let mut result = String::new();

        let mut chars_iter = self.chars.iter().skip(self.index + 1);
        let mut seek = 1;
        while let Some(c1) = chars_iter.next() {
            seek += 1;
            match c1 {
                // end
                '\"' => {
                    self.advance(seek);
                    Self::push_utf16(&mut result, &mut utf16);
                    return Some(Token::String(result));
                }
                // escape
                '\\' => {
                    let c2 = *chars_iter.next()?;
                    seek += 1;

                    if matches!(c2, '"' | '\\' | '/' | 'b' | 'f' | 'n' | 'r' | 't') {
                        Self::push_utf16(&mut result, &mut utf16);
                        result.push('\\');
                        result.push(c2);
                    } else if c2 == 'u' {
                        // UTF-16
                        // \u0000 ~ \uFFFF

                        let mut hexs: Vec<char> = vec![];
                        for _ in 0..4 {
                            let c = chars_iter.next()?;
                            seek += 1;
                            if c.is_ascii_hexdigit() {
                                hexs.push(*c);
                            }
                        }

                        let hex_str: String = hexs.iter().collect();
                        let code_point = u16::from_str_radix(&hex_str, 16).ok()?;
                        utf16.push(code_point);
                    } else {
                        return None;
                    }
                }
                _ => {
                    Self::push_utf16(&mut result, &mut utf16);
                    result.push(*c1);
                }
            }
        }

        None
    }

    fn parse_number(&mut self) -> Option<Token> {
        let c = self.peek()?;
        if !c.is_numeric() && c != '-' {
            return None;
        }

        // parse number
        let number_str = self
            .chars
            .iter()
            .skip(self.index)
            .take_while(|&c| matches!(c, '+' | '-' | 'e' | 'E' | '.') | c.is_numeric())
            .collect::<String>();
        let read_cnt = number_str.len();
        let number = number_str.parse::<f64>().ok()?;
        self.advance(read_cnt);
        Some(Token::Number(number))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_number() {
        //integer
        let num = "1234567890";
        let tokens = Lexer::new(num).lex().unwrap();
        assert_eq!(tokens[0], Token::Number(1234567890f64));

        //float
        let num = "-0.001";
        let tokens = Lexer::new(num).lex().unwrap();
        assert_eq!(tokens[0], Token::Number(-0.001));

        // exponent
        let num = "1e-10";
        let tokens = Lexer::new(num).lex().unwrap();
        assert_eq!(tokens[0], Token::Number(0.0000000001));
    }

    #[test]
    fn test_bool() {
        let b = "true";
        let tokens = Lexer::new(b).lex().unwrap();
        assert_eq!(tokens[0], Token::Bool(true));

        let b = "false";
        let tokens = Lexer::new(b).lex().unwrap();
        assert_eq!(tokens[0], Token::Bool(false));
    }

    #[test]
    fn test_string() {
        let s = "\"togatoga123\"";
        let tokens = Lexer::new(s).lex().unwrap();
        assert_eq!(tokens[0], Token::String("togatoga123".to_string()));

        let s = "\"あいうえお\"";
        let tokens = Lexer::new(s).lex().unwrap();
        assert_eq!(tokens[0], Token::String("あいうえお".to_string()));

        let s = r#""\u3042\u3044\u3046abc""#; //あいうabc

        let tokens = Lexer::new(s).lex().unwrap();
        assert_eq!(tokens[0], Token::String("あいうabc".to_string()));

        let s = format!(r#" " \b \f \n \r \t \/ \" ""#);
        let tokens = Lexer::new(&s).lex().unwrap();
        assert_eq!(
            tokens[0],
            Token::String(r#" \b \f \n \r \t \/ \" "#.to_string())
        );

        let s = r#""\uD83D\uDE04\uD83D\uDE07""#;
        let tokens = Lexer::new(&s).lex().unwrap();
        assert_eq!(tokens[0], Token::String(r#"😄😇"#.to_string()));
    }

    #[test]
    fn test_null() {
        let null = "null";
        let tokens = Lexer::new(null).lex().unwrap();
        assert_eq!(tokens[0], Token::Null);
    }

    #[test]
    fn test_object() {
        let obj = r#"
        {
            "number": 123,
            "boolean": true,
            "string": "togatoga",
            "object": {
               "number": 2E10
            }
         }
         "#;

        let tokens = Lexer::new(obj).lex().unwrap();
        let result_tokens = [
            // start {
            Token::Symbol(Symbol::LeftBrace),
            // begin: "number": 123,
            Token::String("number".to_string()),
            Token::Symbol(Symbol::Colon),
            Token::Number(123f64),
            Token::Symbol(Symbol::Comma),
            // end

            // begin: "boolean": true,
            Token::String("boolean".to_string()),
            Token::Symbol(Symbol::Colon),
            Token::Bool(true),
            Token::Symbol(Symbol::Comma),
            // end

            // begin: "string": "togatoga",
            Token::String("string".to_string()),
            Token::Symbol(Symbol::Colon),
            Token::String("togatoga".to_string()),
            Token::Symbol(Symbol::Comma),
            // end

            // begin: "object": {
            Token::String("object".to_string()),
            Token::Symbol(Symbol::Colon),
            Token::Symbol(Symbol::LeftBrace),
            // begin: "number": 2E10,
            Token::String("number".to_string()),
            Token::Symbol(Symbol::Colon),
            Token::Number(20000000000f64),
            // end
            Token::Symbol(Symbol::RightBrace),
            // end
            Token::Symbol(Symbol::RightBrace),
            // end
        ];
        tokens
            .iter()
            .zip(result_tokens.iter())
            .enumerate()
            .for_each(|(i, (x, y))| {
                assert_eq!(x, y, "index: {}", i);
            });
    }
    #[test]
    fn test_array() {
        let a = "[true, {\"キー\": null}]";
        let tokens = Lexer::new(a).lex().unwrap();
        let result_tokens = vec![
            Token::Symbol(Symbol::LeftBracket),
            Token::Bool(true),
            Token::Symbol(Symbol::Comma),
            Token::Symbol(Symbol::LeftBrace),
            Token::String("キー".to_string()),
            Token::Symbol(Symbol::Colon),
            Token::Null,
            Token::Symbol(Symbol::RightBrace),
            Token::Symbol(Symbol::RightBracket),
        ];
        tokens
            .iter()
            .zip(result_tokens.iter())
            .for_each(|(x, y)| assert_eq!(x, y));
    }
}
