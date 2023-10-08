use crate::{Token, TokenKind};

pub fn tokenize(buffer: &[u8]) -> Vec<Token> {
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

                character += 1;

                if i + 1 < buffer.len() {
                    match TokenKind::from(buffer[i + 1] as char) {
                        TokenKind::Symbol(_) => {}
                        _ => break,
                    }
                }

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

pub fn lex(buffer: &[u8]) -> Vec<Token> {
    filter(&tokenize(buffer))
}

#[cfg(test)]
mod tests {
    use crate::TokenPosition;

    use super::*;

    #[test]
    fn test_tokenize() {
        use TokenKind::*;

        let buffer: &[u8] = b"abcdefghijklmnopqrstuvwxyz0123456789,.\n:/' *=[]{}";

        let expect: Vec<Token> = {
            let kinds: [TokenKind; 14] = [
                Symbol("abcdefghijklmnopqrstuvwxyz0123456789".into()),
                Comma, Dot, Colon, Slash, Quote, Asterisk, Equals,
                OpenBracket, CloseBracket, OpenCurly, CloseCurly,
                Space, NewLine
            ];

            let mut positions: Vec<TokenPosition> = vec![
                TokenPosition { line: 1, character: 0, length: 36 },
                TokenPosition { line: 1, character: 36, length: 1 },
                TokenPosition { line: 1, character: 37, length: 1 },
                TokenPosition { line: 2, character: 0, length: 1 },
                TokenPosition { line: 2, character: 1, length: 1 },
                TokenPosition { line: 2, character: 2, length: 1 },
                TokenPosition { line: 2, character: 3, length: 1 },
                TokenPosition { line: 2, character: 4, length: 1 },
                TokenPosition { line: 2, character: 5, length: 1 },
                TokenPosition { line: 2, character: 6, length: 1 },
                TokenPosition { line: 2, character: 7, length: 1 },
                TokenPosition { line: 2, character: 8, length: 1 },
                TokenPosition { line: 2, character: 9, length: 1 },
                TokenPosition { line: 2, character: 10, length: 1 },
            ].into_iter().rev().collect();

            kinds.into_iter().map(|kind| {
                Token { kind, position: positions.pop().unwrap() }
            }).collect()
        };

        let tokens = tokenize(buffer);

        assert_eq!(tokens, expect);
    }
}