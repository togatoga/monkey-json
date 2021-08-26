use std::collections::BTreeMap;

use lexer::Lexer;
use parser::{Parser, ParserError};

mod lexer;
mod parser;

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Number(f64),
    Bool(bool),
    String(String),
    Null,
    Array(Vec<Value>),
    Object(BTreeMap<String, Value>),
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

pub fn parse(input: &str) -> Result<Value, ParserError> {
    match Lexer::new(input).tokenize() {
        Ok(tokens) => Parser::new(tokens).parse(),
        Err(e) => Err(ParserError::new(&e.msg)),
    }
}
