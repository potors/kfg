use crate::{
    parser::{ParseTokens, ParserError},
    Token, TokenKind,
};
use std::{collections::HashMap, iter::Peekable, slice::Iter};

const INDENT: fn(String) -> String = |s| s.replace('\n', "\n  ");

#[derive(Debug, Default, Clone)]
pub struct Ast(pub HashMap<String, Node>);

impl std::fmt::Display for Ast {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut s = String::new();

        let mut arr = self.0.iter().collect::<Vec<_>>();

        let last = arr.pop();

        for (key, value) in arr {
            s.push_str(&format!("{key}: {value},\n"));
        }

        if let Some((key, value)) = last {
            s.push_str(&format!("{key}: {value}"));
        }

        write!(f, "{{\n  {}\r\n}}", INDENT(s))
    }
}

#[derive(Debug, Clone)]
pub enum Node {
    String(String),
    Integer(i64),
    Float(f64),
    Bool(bool),
    Array(Vec<Node>),
    Dict(HashMap<String, Node>),
    Null,
}

impl std::fmt::Display for Node {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let node = match self {
            Node::String(value) => format!("\x1b[32m\"{value}\"\x1b[m"),
            Node::Integer(value) => format!("\x1b[33m{value}\x1b[m"),
            Node::Float(value) => format!("\x1b[33m{value}\x1b[m"),
            Node::Bool(value) => format!("\x1b[34m{value}\x1b[m"),
            Node::Array(value) => {
                if value.is_empty() {
                    "[]".to_string()
                } else {
                    let initial = INDENT(value[0].to_string());

                    let arr = value
                        .iter()
                        .skip(1)
                        .map(|node| INDENT(format!("{node}")))
                        .fold(initial, |acc, e| format!("{acc},\n  {e}"));

                    format!("[\n  {arr}\n]")
                }
            }
            Node::Dict(value) => {
                let value = value.iter().collect::<Vec<_>>();

                if value.is_empty() {
                    "{}".to_string()
                } else {
                    let initial = INDENT(format!("{}: {}", value[0].0, value[0].1));

                    let dict = value
                        .iter()
                        .skip(1)
                        .map(|(key, value)| INDENT(format!("{key}: {value}")))
                        .fold(initial, |acc, e| format!("{acc},\n  {e}"));

                    format!("{{\n  {dict}\n}}")
                }
            }
            Node::Null => "\x1b[31mnull\x1b[m".into(),
        };

        write!(f, "{node}")
    }
}

impl TryFrom<String> for Node {
    type Error = ParserError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        match value.as_str() {
            "true" => Ok(Self::Bool(true)),
            "false" => Ok(Self::Bool(false)),
            "null" => Ok(Self::Null),
            _ => {
                if let Ok(value) = value.parse::<i64>() {
                    Ok(Self::Integer(value))
                } else if let Ok(value) = value.parse::<f64>() {
                    Ok(Self::Float(value))
                } else {
                    Err(ParserError::InvalidSymbol(value.clone()))
                }
            }
        }
    }
}

impl TryFrom<&str> for Node {
    type Error = ParserError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        value.to_string().try_into()
    }
}

impl TryFrom<&Token> for Node {
    type Error = ParserError;

    fn try_from(value: &Token) -> Result<Self, Self::Error> {
        match value.kind {
            TokenKind::Symbol(ref symbol) => symbol.clone().try_into(),
            _ => Err(ParserError::InvalidToken(value.clone())),
        }
    }
}

impl TryFrom<&mut Peekable<Iter<'_, Token>>> for Node {
    type Error = ParserError;

    fn try_from(iter: &mut Peekable<Iter<'_, Token>>) -> Result<Self, Self::Error> {
        use TokenKind::{OpenBracket, OpenCurly, Quote, Symbol};

        let token = iter.peek().unwrap();

        match token.kind {
            Symbol(_) => iter.parse_symbol(),
            Quote => iter.parse_string(),
            OpenBracket => iter.parse_array(),
            OpenCurly => iter.parse_dict(),
            _ => Err(ParserError::UnreachableToken((*token).clone())),
        }
    }
}
