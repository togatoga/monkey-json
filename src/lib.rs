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
        assert_eq!(self.peek(), Some(&Token::LeftBracket));
        self.advance();
        let mut array = vec![];
        if self.peek() == Some(&Token::RightBracket) {
            self.advance();
            return Ok(Value::Array(array));
        }
        while self.peekable() {
            let value = self.parse().expect("failed to parse a value");
            array.push(value);
            let token = self.peek().unwrap();
            match token {
                Token::RightBracket => {
                    self.advance();
                    return Ok(Value::Array(array));
                }
                Token::Comma => {
                    self.advance();
                }
                _ => {
                    panic!("Expected comma after object in array: {:?}", token);
                }
            }
        }
        Err(ParseError::new("Failed to parse array"))
    }
    fn parse_object(&mut self) -> Result<Value, ParseError> {
        assert_eq!(self.peek(), Some(&Token::LeftBrace));
        self.advance();
        let mut object = BTreeMap::new();
        let token = self.peek();
        if token == Some(&Token::RightBrace) {
            return Ok(Value::Object(object));
        }

        while self.peekable() {
            // {"togatoga": 10}

            // "togatoga"
            let token = self.peek().unwrap().clone();
            let key = match token {
                Token::String(key) => {
                    self.advance();
                    key
                }
                _ => {
                    panic!("Expected string key");
                }
            };
            // :
            if self.peek() == Some(&Token::Colon) {
                self.advance();
            } else {
                panic!("Expected colon after key in object");
            }
            // 10
            let value = self.parse().expect("failed to parse a value");
            object.insert(key.clone(), value);

            // }
            let token = self.peek().unwrap().clone();
            match token {
                Token::RightBrace => {
                    self.advance();
                    return Ok(Value::Object(object));
                }
                Token::Comma => {
                    self.advance();
                }
                _ => {
                    panic!("Expected comma after key-value in obeject: {:?}", token);
                }
            }
        }

        Err(ParseError::new("Failed to parse object"))
    }

    fn parse(&mut self) -> Result<Value, ParseError> {
        if !self.peekable() {
            return Err(ParseError::new("can't peek"));
        }
        let t = self.tokens[self.index].clone();
        if t == Token::LeftBracket {
            // [
            return self.parse_array();
        } else if t == Token::LeftBrace {
            // {
            return self.parse_object();
        }
        let value = match t {
            Token::String(s) => Value::String(s),
            Token::Number(n) => Value::Number(n),
            Token::Bool(b) => Value::Bool(b),
            Token::Null => Value::Null,
            _ => panic!("failed to parse"),
        };
        self.advance();
        Ok(value)
    }
    fn advance(&mut self) {
        self.index += 1;
    }
    fn peek(&self) -> Option<&Token> {
        self.tokens.get(self.index)
    }
    fn peekable(&self) -> bool {
        self.index < self.tokens.len()
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
