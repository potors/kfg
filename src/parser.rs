use crate::{Ast, Node, Token, TokenKind};
use std::{collections::HashMap, iter::Peekable, slice::Iter};

#[derive(Debug, Clone)]
pub enum ParserError {
    MissingValueAfterDeclaration(Token),
    MissingToken(TokenKind, Token),
    MismatchedTokenType(TokenKind, Token),
    InvalidToken(Token),
    InvalidSymbol(String),
    BrokenString(Token),
    UnclosedString(Token),
    TrailingComma(Token),
    ScopeInsideDict(Token),
    UnexpectedEOF(Token),
    UnreachableToken(Token),
}

impl std::error::Error for ParserError {}

impl std::fmt::Display for ParserError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}

pub fn parse(tokens: &[Token]) -> Result<Ast, ParserError> {
    let mut ast = Ast::default();

    let mut scopes: Option<Vec<String>> = None;

    let mut iter = tokens.iter().peekable();

    while let Some(key) = iter.next() {
        use ParserError::*;
        use TokenKind::*;

        match key.kind {
            NewLine => {}
            Symbol(ref symbol) => {
                if iter.peek().is_none() {
                    return Err(UnexpectedEOF(key.clone()));
                }

                let next = iter.next().unwrap();

                match next.kind {
                    Equals => unsafe {
                        if iter.peek().is_none() {
                            return Err(MissingValueAfterDeclaration(key.clone()));
                        }

                        let node = Node::try_from(&mut iter)?;

                        if let Some(ref mut keys) = scopes {
                            keys.push(symbol.clone());

                            let mut root = Node::Dict(HashMap::new());

                            {
                                let mut child = &mut root;
                                let mut scopes = keys.iter().skip(1).peekable();

                                while let Some(scope) = scopes.next() {
                                    if let Node::Dict(ref mut dict) = child {
                                        if scopes.peek().is_some() {
                                            dict.insert(scope.clone(), Node::Dict(HashMap::new()));

                                            child = (&mut *(dict as *mut HashMap<String, Node>))
                                                .get_mut(scope)
                                                .unwrap();
                                        } else {
                                            dict.insert(scope.clone(), node.clone());
                                        }
                                    }
                                }
                            }

                            let mut dict = &mut ast.0;
                            for key in keys {
                                match dict.get_mut(key) {
                                    Some(Node::Dict(inner)) => {
                                        dict = &mut *(inner as *mut _);
                                        continue;
                                    }
                                    _ => {
                                        dict.insert(key.clone(), root);
                                        break;
                                    }
                                }
                            }

                            scopes = None;
                        } else {
                            ast.0.insert(symbol.clone(), node);
                        }
                    },
                    Colon => match iter.next() {
                        Some(next) => match next.kind {
                            Colon => match scopes {
                                Some(ref mut scopes) => scopes.push(symbol.clone()),
                                None => scopes = Some(vec![symbol.clone()]),
                            },
                            _ => return Err(MismatchedTokenType(TokenKind::Colon, key.clone())),
                        },
                        None => return Err(UnexpectedEOF(next.clone())),
                    },
                    _ => return Err(UnreachableToken(key.clone())),
                }
            }
            Dot => return Err(InvalidToken(key.clone())),
            _ => return Err(UnreachableToken(key.clone())),
        }
    }

    Ok(ast)
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

        log::debug!("\x1b[33m*\x1b[m {node:?}");

        Ok(node)
    }

    fn parse_string(&mut self) -> Result<Node, ParserError> {
        let mut s = String::new();

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

            s.push_str(token.kind.as_str());
        }

        let node = Node::String(s);

        log::debug!("\x1b[33m*\x1b[m {node:?}");

        Ok(node)
    }

    fn parse_array(&mut self) -> Result<Node, ParserError> {
        let mut array = Vec::new();

        // skip first open curly
        self.next();

        log::trace!("\x1b[36m* new array\x1b[m");

        while let Some(&token) = self.peek() {
            match token.kind {
                TokenKind::CloseBracket => {
                    self.next();
                    break;
                }
                TokenKind::NewLine | TokenKind::Space | TokenKind::Comma => {
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

        if log::log_enabled!(log::Level::Trace) {
            log::trace!("\x1b[36m* end array\x1b[m");
        } else {
            log::debug!("\x1b[33m*\x1b[m {array:?}");
        }

        Ok(Node::Array(array))
    }

    fn parse_dict(&mut self) -> Result<Node, ParserError> {
        let mut dict = HashMap::new();

        // skip first open curly
        self.next();

        log::trace!("\x1b[35m* new dict\x1b[m");

        while let Some(&token) = self.peek() {
            match token.kind {
                TokenKind::CloseCurly => {
                    self.next();
                    break;
                }
                TokenKind::NewLine | TokenKind::Space | TokenKind::Comma => {
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
                                    Some(&token) => if let TokenKind::Colon = token.kind {
                                        return Err(ParserError::ScopeInsideDict(token.clone()));
                                    }
                                    None => return Err(ParserError::UnexpectedEOF(token.clone())),
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

        if log::log_enabled!(log::Level::Trace) {
            log::trace!("\x1b[35m* end dict\x1b[m");
        } else {
            log::debug!("\x1b[33m*\x1b[m {dict:?}");
        }

        Ok(Node::Dict(dict))
    }
}
