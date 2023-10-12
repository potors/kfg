use crate::{parser::ParserError, Token, TokenKind};
use std::{collections::HashMap, iter::Peekable, slice::Iter};

const INDENT: fn(String) -> String = |s| s.replace('\n', "\n  ");

#[derive(Debug, Default, Clone)]
pub struct Ast(pub HashMap<String, Node>);

impl Ast {
    pub fn assignments(&self) -> usize {
        get_len(&self.0)
    }

    pub fn inline(&self) -> String {
        let string = self.0.iter()
            .fold(String::new(), |acc, (key, value)| {
                format!("{acc}, {key}: {}", value.inline())
            })
            .trim_start_matches(", ")
            .to_string();

        format!("{{{string}}}")
    }
}

fn get_len(dict: &HashMap<String, Node>) -> usize {
    dict.iter().map(|(_, node)| {
        if let Node::Dict(dict) = node {
            get_len(dict)
        } else {
            0
        }
    }).sum::<usize>() + dict.len()
}

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

impl Node {
    pub fn inline(&self) -> String {
        match self {
            Node::Array(value) => {
                if value.is_empty() {
                    "[]".to_string()
                } else {
                    let initial = value[0].inline();

                    let arr = value
                        .iter()
                        .skip(1)
                        .map(|node| node.inline())
                        .fold(initial, |acc, e| format!("{acc}, {e}"));

                    format!("[{arr}]")
                }
            }
            Node::Dict(value) => {
                let value: Vec<(&String, &Node)> = value.iter().collect();

                if value.is_empty() {
                    "{}".to_string()
                } else {
                    let initial = format!("{}: {}", value[0].0, value[0].1.inline());

                    let dict = value
                        .iter()
                        .skip(1)
                        .map(|(key, value)| format!("{key}: {}", value.inline()))
                        .fold(initial, |acc, e| format!("{acc}, {e}"));

                    format!("{{{dict}}}")
                }
            }
            node => node.to_string(),
        }
    }
}

impl std::fmt::Display for Node {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let string = match self {
            Node::String(value) => format!("\x1b[32m{value:?}\x1b[m"),
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

        write!(f, "{string}")
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

        let token = *iter.peek().unwrap();

        let node = match token.kind {
            Symbol(_) => iter.parse_symbol(),
            Quote => iter.parse_string(),
            OpenBracket => iter.parse_array(),
            OpenCurly => iter.parse_dict(),
            _ => Err(ParserError::UnreachableToken(token.clone())),
        };

        if let Ok(node) = &node {
            trace!("{}", node.inline());
        }

        node
    }
}

impl PartialEq for Node {
    fn eq(&self, other: &Self) -> bool {
        use Node::*;
        
        match (self, other) {
            (String(a), String(b)) => a == b,
            (Integer(a), Integer(b)) => a == b,
            (Float(a), Float(b)) => a == b,
            (Bool(a), Bool(b)) => a == b,
            (Array(a), Array(b)) => a == b,
            (Dict(a), Dict(b)) => a == b,
            (Null, Null) => true,
            _ => false,
        }
    }
}

pub trait ParseTokens {
    fn parse_symbol(&mut self) -> Result<Node, ParserError>;
    fn parse_string(&mut self) -> Result<Node, ParserError>;
    fn parse_array(&mut self) -> Result<Node, ParserError>;
    fn parse_dict(&mut self) -> Result<Node, ParserError>;
}

impl ParseTokens for Peekable<Iter<'_, Token>> {
    fn parse_symbol(&mut self) -> Result<Node, ParserError> {
        let token = self.next().unwrap();

        let node = {
            if let Some(&dot) = self.peek() {
                if let TokenKind::Dot = dot.kind {
                    // skip dot
                    self.next();

                    if self.peek().is_none() {
                        return Err(ParserError::MissingToken(
                            TokenKind::Symbol("".into()),
                            dot.clone(),
                        ));
                    }

                    let left = token.kind.as_str();
                    let right = self.next().unwrap().kind.as_str();

                    Some(Node::try_from(format!("{left}.{right}"))?)
                } else {
                    None
                }
            } else {
                None
            }
        }
        .unwrap_or(Node::try_from(token)?);

        Ok(node)
    }

    fn parse_string(&mut self) -> Result<Node, ParserError> {
        let mut string = String::new();

        // skip first quote
        self.next();

        while let Some(token) = self.next() {
            match token.kind {
                TokenKind::Quote => break,
                TokenKind::NewLine => return Err(ParserError::BrokenString(token.clone())),
                _ => {}
            }

            if self.peek().is_none() {
                return Err(ParserError::UnclosedString(token.clone()));
            }

            if let TokenKind::BackSlash = token.kind {
                string.push_str(self.next().unwrap().kind.as_str());
            } else {
                string.push_str(token.kind.as_str());
            }
        }

        Ok(Node::String(string))
    }

    fn parse_array(&mut self) -> Result<Node, ParserError> {
        let mut array = Vec::new();

        // skip first open curly
        self.next();

        while let Some(&token) = self.peek() {
            match token.kind {
                TokenKind::CloseBracket => {
                    self.next();
                    break;
                }
                TokenKind::NewLine | TokenKind::Space | TokenKind::Tab | TokenKind::Comma => {
                    self.next();
                    continue;
                }
                TokenKind::Symbol(_)
                | TokenKind::Quote
                | TokenKind::OpenBracket
                | TokenKind::OpenCurly => {
                    array.push(Node::try_from(&mut *self)?);
                }
                _ => {
                    return Err(ParserError::InvalidToken(token.clone()));
                }
            }
        }

        Ok(Node::Array(array))
    }

    fn parse_dict(&mut self) -> Result<Node, ParserError> {
        let mut dict = HashMap::new();

        // skip first open curly
        self.next();

        while let Some(&token) = self.peek() {
            match token.kind {
                TokenKind::CloseCurly => {
                    self.next();
                    break;
                }
                TokenKind::NewLine | TokenKind::Space | TokenKind::Tab | TokenKind::Comma => {
                    self.next();
                    continue;
                }
                TokenKind::Dot => {
                    self.next();

                    if self.peek().is_none() {
                        return Err(ParserError::UnexpectedEOF(token.clone()));
                    }

                    let token = self.next().unwrap();

                    if let TokenKind::Symbol(ref key) = token.kind {
                        if self.peek().is_none() {
                            return Err(ParserError::UnexpectedEOF(token.clone()));
                        }

                        match self.next() {
                            Some(token) if matches!(token.kind, TokenKind::Colon) => {
                                match self.peek() {
                                    Some(&token) if matches!(token.kind, TokenKind::Colon) => {
                                        return Err(ParserError::ScopeInsideDict(token.clone()));
                                    }
                                    None => return Err(ParserError::UnexpectedEOF(token.clone())),
                                    _ => {}
                                }

                                let node = Node::try_from(&mut *self)?;
                                dict.insert(key.clone(), node);
                            }
                            None => return Err(ParserError::UnexpectedEOF(token.clone())),
                            _ => return Err(ParserError::MismatchedTokenType(TokenKind::Colon, token.clone())),
                        }
                    } else {
                        return Err(ParserError::MismatchedTokenType(TokenKind::Symbol("".into()), token.clone()));
                    }
                }
                _ => {
                    return Err(ParserError::InvalidToken(token.clone()));
                }
            }
        }

        Ok(Node::Dict(dict))
    }
}