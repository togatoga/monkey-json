use crate::{lexer::Token, Value};

#[derive(Debug, Clone)]
pub struct ParserError {
    pub msg: String,
}

impl ParserError {
    pub fn new(msg: &str) -> ParserError {
        ParserError {
            msg: msg.to_string(),
        }
    }
}
pub struct Parser {
    // `Lexer`で`tokenize`した`Token`一覧
    tokens: Vec<Token>,
    // `tokens`の先頭
    index: usize,
}

impl Parser {
    /// `Token`の一覧を受け取り`Parser`を返す。
    pub fn new(tokens: Vec<Token>) -> Parser {
        Parser { tokens, index: 0 }
    }

    /// `Array`のパースを行う。
    ///  [1, 2, 3, null, "string"]
    fn parse_array(&mut self) -> Result<Value, ParserError> {
        // 先頭は必ず [
        let token = self.peek_expect()?;
        if *token != Token::LeftBracket {
            return Err(ParserError::new(&format!(
                "error: JSON array must starts [ {:?}",
                token
            )));
        }
        // [ を読み飛ばす
        self.next_expect()?;

        let mut array = vec![];
        let token = self.peek_expect()?;
        // ] なら空配列を返す
        if *token == Token::RightBracket {
            return Ok(Value::Array(array));
        }

        loop {
            // 残りの`Value`をパースする
            let value = self.parse()?;
            array.push(value);

            // `Array`が終端もしくは次の要素(`Value`)があるかを確認
            let token = self.next_expect()?;
            match token {
                // ] は`Array`の終端
                Token::RightBracket => {
                    return Ok(Value::Array(array));
                }
                // , なら次の要素(`Value`)をパースする
                Token::Comma => {
                    continue;
                }
                _ => {
                    return Err(ParserError::new(&format!(
                        "error: a [ or , token is expected {:?}",
                        token
                    )));
                }
            }
        }
    }

    /// `Object`のパースを行う。
    /// {
    ///   "key1": 12345,
    ///   "key2": 6789
    /// }
    fn parse_object(&mut self) -> Result<Value, ParserError> {
        // 先頭は必ず {
        let token = self.peek_expect()?;
        if *token != Token::LeftBrace {
            return Err(ParserError::new(&format!(
                "error: JSON object must starts {{ {:?}",
                token
            )));
        }
        // { を読み飛ばす
        self.next_expect()?;

        let mut object = std::collections::BTreeMap::new();

        // } なら空の`Object`を返す
        if *self.peek_expect()? == Token::RightBrace {
            return Ok(Value::Object(object));
        }

        loop {
            // 2文字分の`Token`を読み出す
            let token1 = self.next_expect()?.clone();
            let token2 = self.next_expect()?;

            match (token1, token2) {
                // token1とtoken2はそれぞれ"key"(`Token::String`)と:(`Token::Colon`)であることを想定してる
                // e.g. "key" : 12345
                // "key" (`Token::String`)
                // : (`Token::Colon`)
                (Token::String(key), Token::Colon) => {
                    // 残りの`Value`(12345)をパースする。
                    object.insert(key, self.parse()?);
                }
                // それ以外はエラー
                _ => {
                    return Err(ParserError::new(
                        "error: a pair (key(string) and : token) token is expected",
                    ));
                }
            }
            // `Object`が終端かもしくは次の要素(key-value)があるか
            let token3 = self.next_expect()?;
            match token3 {
                // } `Object`の終端だったら`object`を返す
                Token::RightBrace => {
                    return Ok(Value::Object(object));
                }
                // , なら次の要素(key-value)のパースする
                // {
                //    "key1": 12345,
                //    "key2": 6789
                // }
                Token::Comma => {
                    continue;
                }
                _ => {
                    return Err(ParserError::new(&format!(
                        "error: a {{ or , token is expected {:?}",
                        token3
                    )));
                }
            }
        }
    }

    /// `Token`を評価して`Value`に変換する。この関数は再帰的に呼び出される。
    pub fn parse(&mut self) -> Result<Value, ParserError> {
        let token = self.peek_expect()?.clone();
        let value = match token {
            // { はObjectの開始文字
            Token::LeftBrace => self.parse_object(),
            // [ はArrayの開始文字
            Token::LeftBracket => self.parse_array(),
            Token::String(s) => {
                self.next_expect()?;
                Ok(Value::String(s))
            }
            Token::Number(n) => {
                self.next_expect()?;
                Ok(Value::Number(n))
            }
            Token::Bool(b) => {
                self.next_expect()?;
                Ok(Value::Bool(b))
            }
            Token::Null => {
                self.next_expect()?;
                Ok(Value::Null)
            }
            _ => {
                return Err(ParserError::new(&format!(
                    "error: a token must start {{ or [ or string or number or bool or null {:?}",
                    token
                )))
            }
        };
        value
    }
    /// 先頭の`Token`を返す。
    fn peek(&self) -> Option<&Token> {
        self.tokens.get(self.index)
    }

    /// 先頭の`Token`を返す。(先頭に`Token`があることを想定してる)
    fn peek_expect(&self) -> Result<&Token, ParserError> {
        self.peek()
            .ok_or_else(|| ParserError::new("error: a token isn't peekable"))
    }

    /// 先頭の`Token`を返して、1トークン進める。
    fn next(&mut self) -> Option<&Token> {
        self.index += 1;
        self.tokens.get(self.index - 1)
    }

    /// 先頭の`Token`を返して、1トークン進める。(先頭に`Token`があることを想定してる)
    fn next_expect(&mut self) -> Result<&Token, ParserError> {
        self.next()
            .ok_or_else(|| ParserError::new("error: a token isn't peekable"))
    }
}

#[cfg(test)]
mod test {
    use std::collections::BTreeMap;

    use crate::{lexer::Lexer, Value};

    use super::Parser;

    #[test]
    fn test_parse_object() {
        let json = r#"{"togatoga" : "monkey-json"}"#;
        let value = Parser::new(Lexer::new(json).tokenize().unwrap())
            .parse()
            .unwrap();
        let mut object = BTreeMap::new();
        object.insert(
            "togatoga".to_string(),
            Value::String("monkey-json".to_string()),
        );
        assert_eq!(value, Value::Object(object));

        let json = r#"
        {
            "key": {
                "key": false
            }
        }
        "#;

        let value = Parser::new(Lexer::new(json).tokenize().unwrap())
            .parse()
            .unwrap();
        let mut object = BTreeMap::new();
        let mut nested_object = BTreeMap::new();
        nested_object.insert("key".to_string(), Value::Bool(false));
        object.insert("key".to_string(), Value::Object(nested_object));
        assert_eq!(value, Value::Object(object));
    }

    #[test]
    fn test_parse_array() {
        let json = r#"[null, 1, true, "monkey-json"]"#;
        let value = Parser::new(Lexer::new(json).tokenize().unwrap())
            .parse()
            .unwrap();
        let array = Value::Array(vec![
            Value::Null,
            Value::Number(1.0),
            Value::Bool(true),
            Value::String("monkey-json".to_string()),
        ]);
        assert_eq!(value, array);

        let json = r#"[["togatoga", 123]]"#;
        let value = Parser::new(Lexer::new(json).tokenize().unwrap())
            .parse()
            .unwrap();
        let array = Value::Array(vec![Value::Array(vec![
            Value::String("togatoga".to_string()),
            Value::Number(123.0),
        ])]);
        assert_eq!(value, array);
    }

    #[test]
    fn test_parse() {
        let json = r#"{"key" : [1, "value"]}"#;
        let value = Parser::new(Lexer::new(json).tokenize().unwrap())
            .parse()
            .unwrap();
        let mut object = BTreeMap::new();
        object.insert(
            "key".to_string(),
            Value::Array(vec![Value::Number(1.0), Value::String("value".to_string())]),
        );
        assert_eq!(value, Value::Object(object));

        let json = r#"[{"key": "value"}]"#;
        let value = Parser::new(Lexer::new(json).tokenize().unwrap())
            .parse()
            .unwrap();
        let mut object = BTreeMap::new();
        object.insert("key".to_string(), Value::String("value".to_string()));

        let array = Value::Array(vec![Value::Object(object)]);
        assert_eq!(value, array);
    }
}
