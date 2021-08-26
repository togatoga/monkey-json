use std::collections::BTreeMap;

use crate::{lexer::Token, ParserError, Value};

pub struct Parser {
    tokens: Vec<Token>,
    index: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Parser {
        Parser { tokens, index: 0 }
    }
    fn parse_array(&mut self) -> Result<Value, ParserError> {
        //assert_eq!(self.peek(), Some(&Token::LeftBracket));
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
        let mut object = BTreeMap::new();
        let token = self.peek_expect()?;

        if *token == Token::RightBrace {
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
        let token = self
            .next()
            .ok_or_else(|| ParserError::new("error: no token"))?
            .clone();
        match token {
            Token::LeftBrace => self.parse_object(),
            Token::LeftBracket => self.parse_array(),
            Token::String(s) => Ok(Value::String(s)),
            Token::Number(n) => Ok(Value::Number(n)),
            Token::Bool(b) => Ok(Value::Bool(b)),
            Token::Null => Ok(Value::Null),
            _ => {
                return Err(ParserError::new(&format!(
                    "error: a token must start {{ or [ or string or number or bool or null {:?}",
                    token
                )))
            }
        }
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
