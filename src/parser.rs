use crate::{Ast, Node, Token, TokenKind};
use std::collections::HashMap;

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
    EscapeOutsideOfString(Token),
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

                                            child = (*(dict as *mut HashMap<String, Node>))
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
            BackSlash => return Err(EscapeOutsideOfString(key.clone())),
            _ => return Err(UnreachableToken(key.clone())),
        }
    }

    debug!("\x1b[1;33m*\x1b[39m Assignments: \x1b[36m{}\x1b[m", ast.assignments());

    Ok(ast)
}

#[cfg(test)]
#[rustfmt::skip]
mod tests {
    use super::*;

    #[test]
    fn test_parse() {
        use TokenKind::*;

        let tokens: &[Token] = &[
            Symbol("var1".into()), Equals, Symbol("null".into()), NewLine,
            Symbol("var2".into()), Equals, Symbol("1234".into()), NewLine,
            Symbol("var3".into()), Equals, Symbol("01234".into()), NewLine,
            Symbol("var4".into()), Equals, Symbol("12.34".into()), NewLine,
            Symbol("var5".into()), Equals, Symbol("012.34".into()), NewLine,
            Symbol("var6".into()), Equals, Quote, Symbol("str".into()), Quote, NewLine,
            Symbol("var7".into()), Equals, Quote, Symbol(" s  t  r ".into()), Quote, NewLine,
            Symbol("var8".into()), Equals, Symbol("true".into()), NewLine,
            Symbol("var9".into()), Equals, Symbol("false".into()), NewLine,
            Symbol("var0".into()), Equals, OpenBracket, Symbol("null".into()), Comma, Symbol("null".into()), CloseBracket, NewLine,
            Symbol("vara".into()), Equals, OpenCurly, Dot, Symbol("entry".into()), Colon, Symbol("null".into()), CloseCurly, NewLine,
            Symbol("varb".into()), Colon, Colon, Symbol("nested".into()), Equals, Symbol("null".into()), NewLine,
        ].map(|kind| Token::new(kind, (0, 0, 0)));

        let expected = HashMap::<String, _>::from([
            ("var1".into(), Node::Null),
            ("var2".into(), Node::Integer(1234)),
            ("var3".into(), Node::Integer(1234)),
            ("var4".into(), Node::Float(12.34)),
            ("var5".into(), Node::Float(12.34)),
            ("var6".into(), Node::String("str".into())),
            ("var7".into(), Node::String(" s  t  r ".into())),
            ("var8".into(), Node::Bool(true)),
            ("var9".into(), Node::Bool(false)),
            ("var0".into(), Node::Array(vec![Node::Null, Node::Null])),
            ("vara".into(), Node::Dict(HashMap::from([("entry".to_string(), Node::Null)]))),
            ("varb".into(), Node::Dict(HashMap::from([("nested".into(), Node::Null)]))),
        ]);

        assert_eq!(parse(tokens).unwrap().0, expected);
    }
}
