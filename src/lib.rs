use std::collections::BTreeMap;

use lexer::Lexer;
use parser::Parser;

mod lexer;
mod parser;

#[derive(Debug, Clone)]
pub struct ParserError {
    msg: String,
}

impl ParserError {
    fn new(msg: &str) -> ParserError {
        ParserError {
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

pub fn parse(input: &str) -> Result<Value, ParserError> {
    match Lexer::new(input).tokenize() {
        Ok(tokens) => Parser::new(tokens).parse(),
        Err(e) => Err(ParserError::new(&e.msg)),
    }
}
