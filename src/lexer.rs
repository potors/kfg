use crate::{Token, TokenKind};

pub fn tokenize(buffer: &Vec<u8>) -> Vec<Token> {
    let mut tokens: Vec<Token> = vec![];
    let [mut line, mut character] = [1, 0];

    let mut token = Token::default();

    let mut i = 0;
    while i < buffer.len() {
        let char = buffer[i] as char;

        if char == '\n' {
            line += 1;
            character = 0;
        }

        token.position.line = line;
        token.position.character = character;

        token.kind = char.into();

        if let TokenKind::Symbol(ref mut s) = token.kind {
            s.clear();

            loop {
                s.push(buffer[i].into());

                if i + 1 < buffer.len() {
                    match TokenKind::from(buffer[i + 1] as char) {
                        TokenKind::Symbol(_) => {}
                        _ => break,
                    }
                }

                character += 1;
                i += 1;

                // Reached EOF
                if i == buffer.len() {
                    break;
                }
            }
        } else {
            character += 1;
        }

        token.position.length = character - token.position.character;
        tokens.push(token.clone());

        i += 1;
    }

    tokens
}

enum Ignore {
    Comment,
    CommentBlock,
    String,
    None,
}

pub fn filter(tokens: &[Token]) -> Vec<Token> {
    use TokenKind::{Asterisk, NewLine, Quote, Slash, Space};

    let mut array: Vec<Token> = vec![];

    let mut ignore = Ignore::None;

    for (i, token) in tokens.iter().enumerate() {
        if let Ignore::Comment = ignore {
            if let NewLine = token.kind {
                ignore = Ignore::None;
            }

            continue;
        } else if let Ignore::CommentBlock = ignore {
            if let Slash = token.kind {
                if let Asterisk = tokens[i - 1].kind {
                    match tokens[i - 2].kind {
                        Slash => {}
                        _ => ignore = Ignore::None,
                    }

                    continue;
                }
            }
        } else if let Slash = token.kind {
            match tokens[i + 1].kind {
                Slash => ignore = Ignore::Comment,
                Asterisk => ignore = Ignore::CommentBlock,
                _ => unreachable!(),
            }

            continue;
        } else if let Quote = token.kind {
            ignore = match ignore {
                Ignore::None => Ignore::String,
                Ignore::String => Ignore::None,
                _ => ignore,
            }
        }

        if let Ignore::None = ignore {
            match token.kind {
                Space => {}
                _ => array.push(token.clone()),
            }
        } else if let Ignore::String = ignore {
            array.push(token.clone())
        }
    }

    array
}

pub fn lex(buffer: &Vec<u8>) -> Vec<Token> {
    filter(&tokenize(buffer))
}
