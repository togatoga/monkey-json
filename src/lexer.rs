use std::{iter::Peekable, str::Chars};

#[derive(Debug, PartialEq, Clone)]
pub enum Token {
    String(String), // 文字列
    Number(f64),    // 数値
    Bool(bool),     // 真偽値
    Null,           // Null
    WhiteSpace,     //
    LeftBrace,      // {　JSON object 開始文字
    RightBrace,     // }　JSON object 終了文字
    LeftBracket,    // [　JSON array  開始文字
    RightBracket,   // ]　JSON array  終了文字
    Comma,          // ,　JSON value  区切り文字
    Colon,          // :　"key":value 区切り文字
}

pub struct Lexer<'a> {
    pub chars: Peekable<Chars<'a>>,
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

impl<'a> Lexer<'a> {
    pub fn new(input: &str) -> Lexer {
        Lexer {
            chars: input.chars().peekable(),
        }
    }

    fn consume_return_token(&mut self, token: Token) -> Option<Token> {
        self.chars.next();
        Some(token)
    }

    fn next_token(&mut self) -> Result<Option<Token>, LexerError> {
        match self.chars.peek() {
            Some(c) => match c {
                c if c.is_whitespace() || *c == '\n' => {
                    Ok(self.consume_return_token(Token::WhiteSpace))
                }
                '{' => Ok(self.consume_return_token(Token::LeftBrace)),
                '}' => Ok(self.consume_return_token(Token::RightBrace)),
                '[' => Ok(self.consume_return_token(Token::LeftBracket)),
                ']' => Ok(self.consume_return_token(Token::RightBracket)),
                ',' => Ok(self.consume_return_token(Token::Comma)),
                ':' => Ok(self.consume_return_token(Token::Colon)),
                // "togatoga"
                '"' => {
                    // parse string
                    self.chars.next();
                    self.parse_string_token()
                }
                // -1235
                c if c.is_numeric() || matches!(c, '+' | '-' | '.') => self.parse_number_token(),
                // true
                't' => self.parse_bool_token(true),
                // false
                'f' => self.parse_bool_token(false),
                // null
                'n' => self.parse_null_token(),
                _ => Ok(None),
            },
            None => Ok(None),
        }
    }

    pub fn lex(&mut self) -> Result<Vec<Token>, LexerError> {
        let mut tokens = vec![];

        while let Some(token) = self.next_token()? {
            match token {
                Token::WhiteSpace => {}
                _ => {
                    tokens.push(token);
                }
            }
        }

        Ok(tokens)
    }

    fn parse_null_token(&mut self) -> Result<Option<Token>, LexerError> {
        let s = (0..4).filter_map(|_| self.chars.next()).collect::<String>();

        if s == "null" {
            Ok(Some(Token::Null))
        } else {
            Err(LexerError::new(&format!(
                "error: a null value is expected {}",
                s
            )))
        }
    }

    fn parse_bool_token(&mut self, b: bool) -> Result<Option<Token>, LexerError> {
        if b {
            let s = (0..4).filter_map(|_| self.chars.next()).collect::<String>();
            if s == "true" {
                Ok(Some(Token::Bool(true)))
            } else {
                Err(LexerError::new(&format!(
                    "error: a boolean true is expected {}",
                    s
                )))
            }
        } else {
            let s = (0..5).filter_map(|_| self.chars.next()).collect::<String>();

            if s == "false" {
                Ok(Some(Token::Bool(false)))
            } else {
                Err(LexerError::new(&format!(
                    "error: a boolean false is expected {}",
                    s
                )))
            }
        }
    }
    fn parse_number_token(&mut self) -> Result<Option<Token>, LexerError> {
        // parse number
        let mut number_str = String::new();
        while let Some(&c) = self.chars.peek() {
            if c.is_numeric() | matches!(c, '+' | '-' | 'e' | 'E' | '.') {
                self.chars.next();
                number_str.push(c);
            } else {
                break;
            }
        }

        match number_str.parse::<f64>() {
            Ok(number) => Ok(Some(Token::Number(number))),
            Err(e) => Err(LexerError::new(&format!("error: {}", e.to_string()))),
        }
    }

    fn push_utf16(result: &mut String, utf16: &mut Vec<u16>) -> Result<(), LexerError> {
        if utf16.is_empty() {
            return Ok(());
        }
        match String::from_utf16(utf16) {
            Ok(utf16_str) => {
                result.push_str(&utf16_str);
                utf16.clear();
            }
            Err(e) => {
                return Err(LexerError::new(&format!("error: {}", e.to_string())));
            }
        };
        Ok(())
    }
    fn parse_string_token(&mut self) -> Result<Option<Token>, LexerError> {
        let mut utf16 = vec![];
        let mut result = String::new();
        while let Some(c1) = self.chars.next() {
            match c1 {
                // end
                '\"' => {
                    Self::push_utf16(&mut result, &mut utf16)?;
                    return Ok(Some(Token::String(result)));
                }
                // escape
                '\\' => {
                    let c2 = self
                        .chars
                        .next()
                        .ok_or_else(|| LexerError::new("error: a next char is expected"))?;
                    if matches!(c2, '"' | '\\' | '/' | 'b' | 'f' | 'n' | 'r' | 't') {
                        Self::push_utf16(&mut result, &mut utf16)?;
                        result.push('\\');
                        result.push(c2);
                    } else if c2 == 'u' {
                        // UTF-16
                        // \u0000 ~ \uFFFF
                        let hexs = (0..4)
                            .filter_map(|_| {
                                let c = self.chars.next()?;
                                if c.is_ascii_hexdigit() {
                                    Some(c)
                                } else {
                                    None
                                }
                            })
                            .collect::<Vec<_>>();

                        match u16::from_str_radix(&hexs.iter().collect::<String>(), 16) {
                            Ok(code_point) => utf16.push(code_point),
                            Err(e) => {
                                return Err(LexerError::new(&format!(
                                    "error: a unicode character is expected {}",
                                    e.to_string()
                                )))
                            }
                        };
                    } else {
                        return Err(LexerError::new(&format!(
                            "error: an unexpected escaped char {}",
                            c2
                        )));
                    }
                }
                _ => {
                    Self::push_utf16(&mut result, &mut utf16)?;
                    result.push(c1);
                }
            }
        }
        Ok(None)
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

        let s = r#""\uD83D\uDE04\uD83D\uDE07\uD83D\uDC7A""#;
        let tokens = Lexer::new(&s).lex().unwrap();
        assert_eq!(tokens[0], Token::String(r#"😄😇👺"#.to_string()));
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
            Token::LeftBrace,
            // begin: "number": 123,
            Token::String("number".to_string()),
            Token::Colon,
            Token::Number(123f64),
            Token::Comma,
            // end

            // begin: "boolean": true,
            Token::String("boolean".to_string()),
            Token::Colon,
            Token::Bool(true),
            Token::Comma,
            // end

            // begin: "string": "togatoga",
            Token::String("string".to_string()),
            Token::Colon,
            Token::String("togatoga".to_string()),
            Token::Comma,
            // end

            // begin: "object": {
            Token::String("object".to_string()),
            Token::Colon,
            Token::LeftBrace,
            // begin: "number": 2E10,
            Token::String("number".to_string()),
            Token::Colon,
            Token::Number(20000000000f64),
            // end
            Token::RightBrace,
            // end
            Token::RightBrace,
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
            Token::LeftBracket,
            Token::Bool(true),
            Token::Comma,
            Token::LeftBrace,
            Token::String("キー".to_string()),
            Token::Colon,
            Token::Null,
            Token::RightBrace,
            Token::RightBracket,
        ];
        tokens
            .iter()
            .zip(result_tokens.iter())
            .for_each(|(x, y)| assert_eq!(x, y));
    }
}
