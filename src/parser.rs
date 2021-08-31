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
    tokens: Vec<Token>,
    index: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Parser {
        Parser { tokens, index: 0 }
    }
    fn parse_array(&mut self) -> Result<Value, ParserError> {
        let token = self.peek_expect()?;
        if *token != Token::LeftBracket {
            return Err(ParserError::new(&format!(
                "error: JSON array must starts [ {:?}",
                token
            )));
        }
        self.next_expect()?;

        let mut array = vec![];
        let token = self.peek_expect()?;

        if *token == Token::RightBracket {
            return Ok(Value::Array(array));
        }

        loop {
            let value = self.parse()?;
            array.push(value);
            let token = self.next_expect()?;

            match token {
                Token::RightBracket => {
                    return Ok(Value::Array(array));
                }
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
    fn parse_object(&mut self) -> Result<Value, ParserError> {
        let token = self.peek_expect()?;
        if *token != Token::LeftBrace {
            return Err(ParserError::new(&format!(
                "error: JSON object must starts {{ {:?}",
                token
            )));
        }
        self.next_expect()?;

        let mut object = std::collections::BTreeMap::new();

        if *self.peek_expect()? == Token::RightBrace {
            return Ok(Value::Object(object));
        }

        loop {
            // "togatoga" : [1, 2, 3, 4]
            let token1 = self.next_expect()?.clone();

            let token2 = self.next_expect()?;

            match (token1, token2) {
                (Token::String(key), Token::Colon) => {
                    object.insert(key, self.parse()?);
                }
                _ => {
                    return Err(ParserError::new(
                        "error: a pair (key(string) and : token) token is expected",
                    ));
                }
            }
            let token3 = self.next_expect()?;
            match token3 {
                Token::RightBrace => {
                    return Ok(Value::Object(object));
                }
                Token::Comma => {
                    // next token
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

    pub fn parse(&mut self) -> Result<Value, ParserError> {
        let token = self.peek_expect()?.clone();
        let value = match token {
            Token::LeftBrace => self.parse_object(),
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

    fn peek(&self) -> Option<&Token> {
        self.tokens.get(self.index)
    }
    fn peek_expect(&self) -> Result<&Token, ParserError> {
        self.peek()
            .ok_or_else(|| ParserError::new("error: a token isn't peekable"))
    }
    fn next(&mut self) -> Option<&Token> {
        self.index += 1;
        self.tokens.get(self.index - 1)
    }
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
