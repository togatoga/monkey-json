use std::collections::BTreeMap;
mod lexer;
use lexer::*;

#[derive(Debug, Clone)]
pub struct ParseError {
    msg: String,
}

impl ParseError {
    fn new(msg: &str) -> ParseError {
        ParseError {
            msg: msg.to_string(),
        }
    }
}

#[derive(Debug, Clone)]
pub enum Value {
    Number(f64),
    Bool(bool),
    String(String),
    Null,
    Array(Vec<Value>),
    Object(BTreeMap<String, Value>),
}

impl std::cmp::PartialEq<bool> for Value {
    fn eq(&self, other: &bool) -> bool {
        match self {
            Value::Bool(b) => b == other,
            _ => {
                panic!("A value is not bool");
            }
        }
    }
}

impl std::ops::Index<&str> for Value {
    type Output = Value;
    fn index(&self, key: &str) -> &Self::Output {
        match self {
            Value::Object(map) => map
                .get(key)
                .unwrap_or_else(|| panic!("A key is not found: {}", key)),
            _ => {
                panic!("A value is not object");
            }
        }
    }
}

impl std::ops::Index<usize> for Value {
    type Output = Value;
    fn index(&self, idx: usize) -> &Self::Output {
        match self {
            Value::Array(array) => &array[idx],
            _ => {
                panic!("A value is not array");
            }
        }
    }
}

impl Value {
    pub fn keys(&self) -> Vec<String> {
        match self {
            Value::Object(map) => {
                let keys = map.keys().into_iter().cloned().collect();
                keys
            }
            _ => {
                panic!("A value is not object");
            }
        }
    }
    pub fn iter(&self) -> std::slice::Iter<Value> {
        match self {
            Value::Array(array) => array.iter(),
            _ => {
                panic!("A value is not array");
            }
        }
    }
    pub fn iter_mut(&mut self) -> std::slice::IterMut<Value> {
        match self {
            Value::Array(array) => array.iter_mut(),
            _ => {
                panic!("A value is not array");
            }
        }
    }
}

struct Parser {
    tokens: Vec<Token>,
    index: usize,
}

impl Parser {
    fn new(tokens: Vec<Token>) -> Parser {
        Parser { tokens, index: 0 }
    }
    fn parse_array(&mut self) -> Result<Value, ParseError> {
        //assert_eq!(self.peek(), Some(&Token::LeftBracket));
        let mut array = vec![];
        let token = self
            .peek()
            .ok_or_else(|| ParseError::new("error: a token isn't peekable"))?;
        if *token == Token::RightBracket {
            return Ok(Value::Array(array));
        }

        loop {
            let value = self.parse()?;
            array.push(value);
            let token = self
                .next()
                .ok_or_else(|| ParseError::new("error: a token isn't peekable"))?;

            match token {
                Token::RightBracket => {
                    return Ok(Value::Array(array));
                }
                Token::Comma => {
                    continue;
                }
                _ => {
                    return Err(ParseError::new(&format!(
                        "error: a [ or , token is expected {:?}",
                        token
                    )));
                }
            }
        }
    }
    fn parse_object(&mut self) -> Result<Value, ParseError> {
        let mut object = BTreeMap::new();
        let token = self
            .peek()
            .ok_or_else(|| ParseError::new("error: a token isn't peekable"))?;

        if *token == Token::RightBrace {
            return Ok(Value::Object(object));
        }

        loop {
            // "togatoga" : [1, 2, 3, 4]
            let token1 = self
                .next()
                .ok_or_else(|| ParseError::new("error: a token isn't peekable"))?
                .clone();

            let token2 = self
                .next()
                .ok_or_else(|| ParseError::new("error: a token isn't peekable"))?;
            match (token1, token2) {
                (Token::String(key), Token::Colon) => {
                    object.insert(key, self.parse()?);
                }
                _ => {
                    return Err(ParseError::new(
                        "error: a pair (key(string) and : token) token is expected",
                    ));
                }
            }
            let token3 = self.next().unwrap().clone();
            match token3 {
                Token::RightBrace => {
                    return Ok(Value::Object(object));
                }
                Token::Comma => {
                    // next token
                    continue;
                }
                _ => {
                    return Err(ParseError::new(&format!(
                        "error: a {{ or , token is expected {:?}",
                        token3
                    )));
                }
            }
        }
    }

    fn parse(&mut self) -> Result<Value, ParseError> {
        let token = self
            .next()
            .ok_or_else(|| ParseError::new("error: no token"))?
            .clone();
        match token {
            Token::LeftBrace => self.parse_object(),
            Token::LeftBracket => self.parse_array(),
            Token::String(s) => Ok(Value::String(s)),
            Token::Number(n) => Ok(Value::Number(n)),
            Token::Bool(b) => Ok(Value::Bool(b)),
            Token::Null => Ok(Value::Null),
            _ => {
                return Err(ParseError::new(&format!(
                    "error: a token must start {{ or [ or string or number or bool or null {:?}",
                    token
                )))
            }
        }
    }
    fn peek(&mut self) -> Option<&Token> {
        self.tokens.get(self.index)
    }
    fn next(&mut self) -> Option<&Token> {
        self.index += 1;
        self.tokens.get(self.index - 1)
    }
}

pub fn parse(input: &str) -> Result<Value, ParseError> {
    let mut lexer = Lexer::new(input);
    match lexer.tokenize() {
        Ok(tokens) => {
            let mut parser = Parser::new(tokens);
            parser.parse()
        }
        Err(e) => Err(ParseError::new(&e.msg)),
    }
}
