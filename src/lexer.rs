#[derive(Debug, PartialEq, Clone)]
pub enum Token {
    String(String), // 文字列
    Number(f64),    // 数値
    Bool(bool),     // 真偽値
    Null,           // Null
    WhiteSpace,     // 空白
    LeftBrace,      // {　JSON object 開始文字
    RightBrace,     // }　JSON object 終了文字
    LeftBracket,    // [　JSON array  開始文字
    RightBracket,   // ]　JSON array  終了文字
    Comma,          // ,　JSON value  区切り文字
    Colon,          // :　"key":value 区切り文字
}

/// JSONの文字列をParseして`Token`単位に分割
pub struct Lexer<'a> {
    /// 読み込み中の先頭文字列を指す
    chars: std::iter::Peekable<std::str::Chars<'a>>,
}

/// 字句解析中に発生したエラー
#[derive(Debug)]
pub struct LexerError {
    /// エラーメッセージ
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
    /// 文字列を受け取りLexerを返す
    pub fn new(input: &str) -> Lexer {
        Lexer {
            chars: input.chars().peekable(),
        }
    }
    /// 文字列をToken単位に分割をする
    pub fn tokenize(&mut self) -> Result<Vec<Token>, LexerError> {
        let mut tokens = vec![];
        while let Some(token) = self.next_token()? {
            match token {
                // 空白は今回は捨てるがデバッグ情報として使える(行、列)
                Token::WhiteSpace => {}
                _ => {
                    tokens.push(token);
                }
            }
        }
        Ok(tokens)
    }

    /// 一文字分だけ読み進め、tokenを返す
    fn next_return_token(&mut self, token: Token) -> Option<Token> {
        self.chars.next();
        Some(token)
    }

    /// 文字列を読み込み、マッチしたTokenを返す
    fn next_token(&mut self) -> Result<Option<Token>, LexerError> {
        // 先頭の文字列を読み込む
        match self.chars.peek() {
            Some(c) => match c {
                // 一文字分だけ読み進め、Tokenを返す
                // WhiteSpaceは' 'もしくは'\n'
                c if c.is_whitespace() || *c == '\n' => {
                    Ok(self.next_return_token(Token::WhiteSpace))
                }
                '{' => Ok(self.next_return_token(Token::LeftBrace)),
                '}' => Ok(self.next_return_token(Token::RightBrace)),
                '[' => Ok(self.next_return_token(Token::LeftBracket)),
                ']' => Ok(self.next_return_token(Token::RightBracket)),
                ',' => Ok(self.next_return_token(Token::Comma)),
                ':' => Ok(self.next_return_token(Token::Colon)),

                // Note
                // 以下のマッチ条件は開始文字が該当するTokenの開始文字なら、Tokenの文字列分だけ読み進める

                // Stringは開始文字列 '"'
                // e.g. "togatoga"
                '"' => {
                    // parse string
                    self.chars.next();
                    self.parse_string_token()
                }
                // Numberは開始文字が[0-9]もしくは('+', '-', '.')
                // e.g.
                //     -1235
                //     +10
                //     .00001
                c if c.is_numeric() || matches!(c, '+' | '-' | '.') => self.parse_number_token(),
                // Booleanの"true"の開始文字は 't'
                // e.g.
                //     true
                't' => self.parse_bool_token(true),
                // Boolean("false")の開始文字は't'
                // e.g.
                //     false
                'f' => self.parse_bool_token(false),
                // Nullの開始文字は'n'
                // e.g.
                //     null
                'n' => self.parse_null_token(),
                // 上のルールにマッチしない文字はエラー
                _ => Err(LexerError::new(&format!("error: an unexpected char {}", c))),
            },
            None => Ok(None),
        }
    }

    /// nullの文字列をparseする
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
    /// (true|false)の文字列をparseする
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

    /// 数字として使用可能な文字まで読み込む。読み込んだ文字列が数字(`f64`)としてParseに成功した場合Tokenを返す。
    fn parse_number_token(&mut self) -> Result<Option<Token>, LexerError> {
        let mut number_str = String::new();
        while let Some(&c) = self.chars.peek() {
            // 数字に使いそうな文字は全て読み込む
            // 1e10, 1E10, 1.0000
            if c.is_numeric() | matches!(c, '+' | '-' | 'e' | 'E' | '.') {
                self.chars.next();
                number_str.push(c);
            } else {
                break;
            }
        }

        // 読み込んだ文字列が`f64`としてparse出来た場合、Tokenを返す
        match number_str.parse::<f64>() {
            Ok(number) => Ok(Some(Token::Number(number))),
            Err(e) => Err(LexerError::new(&format!("error: {}", e.to_string()))),
        }
    }

    /// 終端文字'\"'まで文字列を読み込む。UTF-16(\u0000~\uFFFF)や特殊なエスケープ文字(e.g. '\t','\n')も考慮する
    fn parse_string_token(&mut self) -> Result<Option<Token>, LexerError> {
        let mut utf16 = vec![];
        let mut result = String::new();

        while let Some(c1) = self.chars.next() {
            match c1 {
                // Escapeの開始文字'\\'
                '\\' => {
                    // 次の文字を読み込む
                    let c2 = self
                        .chars
                        .next()
                        .ok_or_else(|| LexerError::new("error: a next char is expected"))?;
                    if matches!(c2, '"' | '\\' | '/' | 'b' | 'f' | 'n' | 'r' | 't') {
                        // 特殊なエスケープ文字列の処理
                        // https://www.rfc-editor.org/rfc/rfc8259#section-7
                        // utf16のバッファを文字列にpushしておく
                        Self::push_utf16(&mut result, &mut utf16)?;
                        // 今回はエスケープ処理はせずに入力のまま保存しておく
                        result.push('\\');
                        result.push(c2);
                    } else if c2 == 'u' {
                        // UTF-16
                        // \u0000 ~ \uFFFF
                        // \uまで読み込んだので残りの0000~XXXXの4文字を読み込む
                        // UTF-16に関してはエスケープ処理を行う
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

                        // 読み込んだ文字列を16進数として評価しutf16のバッファにpushしておく
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
                // 文字列の終端'"'
                '\"' => {
                    // utf16のバッファを文字列にpushしておく
                    Self::push_utf16(&mut result, &mut utf16)?;
                    return Ok(Some(Token::String(result)));
                }
                // それ以外の文字列
                _ => {
                    // utf16のバッファを文字列にpushしておく
                    Self::push_utf16(&mut result, &mut utf16)?;
                    result.push(c1);
                }
            }
        }
        Ok(None)
    }

    /// utf16のバッファが存在するならば連結しておく
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
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_number() {
        //integer
        let num = "1234567890";
        let tokens = Lexer::new(num).tokenize().unwrap();
        assert_eq!(tokens[0], Token::Number(1234567890f64));

        //float
        let num = "-0.001";
        let tokens = Lexer::new(num).tokenize().unwrap();
        assert_eq!(tokens[0], Token::Number(-0.001));

        // exponent
        let num = "1e-10";
        let tokens = Lexer::new(num).tokenize().unwrap();
        assert_eq!(tokens[0], Token::Number(0.0000000001));
    }

    #[test]
    fn test_bool() {
        let b = "true";
        let tokens = Lexer::new(b).tokenize().unwrap();
        assert_eq!(tokens[0], Token::Bool(true));

        let b = "false";
        let tokens = Lexer::new(b).tokenize().unwrap();
        assert_eq!(tokens[0], Token::Bool(false));
    }

    #[test]
    fn test_string() {
        let s = "\"togatoga123\"";
        let tokens = Lexer::new(s).tokenize().unwrap();
        assert_eq!(tokens[0], Token::String("togatoga123".to_string()));

        let s = "\"あいうえお\"";
        let tokens = Lexer::new(s).tokenize().unwrap();
        assert_eq!(tokens[0], Token::String("あいうえお".to_string()));

        let s = r#""\u3042\u3044\u3046abc""#; //あいうabc

        let tokens = Lexer::new(s).tokenize().unwrap();
        assert_eq!(tokens[0], Token::String("あいうabc".to_string()));

        let s = format!(r#" " \b \f \n \r \t \/ \" ""#);
        let tokens = Lexer::new(&s).tokenize().unwrap();
        assert_eq!(
            tokens[0],
            Token::String(r#" \b \f \n \r \t \/ \" "#.to_string())
        );

        let s = r#""\uD83D\uDE04\uD83D\uDE07\uD83D\uDC7A""#;
        let tokens = Lexer::new(&s).tokenize().unwrap();
        assert_eq!(tokens[0], Token::String(r#"😄😇👺"#.to_string()));
    }

    #[test]
    fn test_null() {
        let null = "null";
        let tokens = Lexer::new(null).tokenize().unwrap();
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

        let tokens = Lexer::new(obj).tokenize().unwrap();
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
        let tokens = Lexer::new(a).tokenize().unwrap();
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
