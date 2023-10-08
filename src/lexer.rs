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
    use super::*;

    #[test]
    fn test_tokenize() {
        use TokenKind::*;

        let buffer: &[u8] = b"abcdefghijklmnopqrstuvwxyz0123456789,.\n:/' *=[]{}";

        let expect: [Token; 14] = [
                Token::new(Symbol("abcdefghijklmnopqrstuvwxyz0123456789".into()), (1, 0, 36)),
                Token::new(Comma, (1, 36, 1)),
                Token::new(Dot, (1, 37, 1)),
                Token::new(NewLine, (2, 0, 1)),
                Token::new(Colon, (2, 1, 1)),
                Token::new(Slash, (2, 2, 1)),
                Token::new(Quote, (2, 3, 1)),
                Token::new(Space, (2, 4, 1)),
                Token::new(Asterisk, (2, 5, 1)),
                Token::new(Equals, (2, 6, 1)),
                Token::new(OpenBracket, (2, 7, 1)),
                Token::new(CloseBracket, (2, 8, 1)),
                Token::new(OpenCurly, (2, 9, 1)),
                Token::new(CloseCurly, (2, 10, 1)),
        ];

        let tokens = tokenize(buffer);

        assert_eq!(tokens, expect);
    }

    #[test]
    fn test_filter() {
        use TokenKind::*;
        
        let tokens: [Token; 15] = [
            Token::new(Slash, (1, 0, 1)),
            Token::new(Slash, (1, 1, 1)),
            Token::new(Symbol("comment".into()), (1, 2, 7)),
            Token::new(NewLine, (2, 0, 1)),
            Token::new(Symbol("var".into()), (2, 1, 3)),
            Token::new(Equals, (2, 4, 1)),
            Token::new(Symbol("null".into()), (2, 5, 4)),
            Token::new(NewLine, (3, 0, 1)),
            Token::new(Slash, (3, 1, 1)),
            Token::new(Asterisk, (3, 2, 1)),
            Token::new(Symbol("comment".into()), (3, 3, 7)),
            Token::new(Space, (3, 10, 1)),
            Token::new(Symbol("block".into()), (3, 11, 5)),
            Token::new(Asterisk, (3, 16, 1)),
            Token::new(Slash, (3, 17, 1)),
        ];
        
        let expect: &[Token] = &tokens[4..8];
        
        let lexed = filter(&tokens);
        
        assert_eq!(lexed, expect);
    }
}