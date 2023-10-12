use crate::{Token, TokenKind};

pub fn tokenize(buffer: &[u8]) -> Vec<Token> {
    let mut tokens: Vec<Token> = vec![];

    let mut iter = buffer.into_iter().map(|&byte| byte as char).peekable();

    while let Some(char) = iter.next() {
        let mut token = Token::default();

        token.kind = char.into();
        token.position.length = 1;

        match tokens.last() {
            Some(last) if matches!(last.kind, TokenKind::NewLine) => {
                token.position.line += 1;
            }
            Some(last) => {
                token.position.line = last.position.line;
                token.position.character = last.position.character + last.position.length;
            }
            None => {}
        }

        if let TokenKind::Symbol(ref mut symbol) = token.kind {
            while let Some(char) = iter.next() {
                symbol.push(char);
                token.position.length += 1;

                if !matches!(iter.peek().map(|&char| TokenKind::from(char)), Some(TokenKind::Symbol(_))) {
                    break;
                }
            }
        }

        tokens.push(token);
    }

    debug!("\x1b[1;33m*\x1b[37m Tokens: \x1b[36m{}\x1b[m", tokens.len());
    tokens
}

pub fn filter(tokens: &[Token]) -> Vec<Token> {
    use TokenKind::*;

    let mut array: Vec<Token> = vec![];

    let mut iter = tokens.iter().peekable();

    while let Some(token) = iter.next() {
        let next = iter.peek().map(|&token| &token.kind);

        match (&token.kind, &next) {
            // Comment
            (Slash, Some(Slash)) => {
                while iter.next().is_some_and(|token| !matches!(token.kind, NewLine)) {}

                let from = token.position;
                trace!("\x1b[31m-\x1b[m \x1b[35mComment\x1b[m from \x1b[36m{}:{}\x1b[m to \x1b[36m{}:{}\x1b[m", from.line, from.character, from.line + 1, 0);

                continue;
            }
            // Comment block
            (Slash, Some(Asterisk)) => {
                while !matches!(
                    (iter.next(), iter.peek()),
                    (Some(left), Some(&right))
                        if matches!(left.kind, Asterisk)
                        && matches!(right.kind, Slash)
                ) {}

                let from = token.position;
                let to = iter.next().unwrap().position;
                trace!("\x1b[31m-\x1b[m \x1b[35mComment Block\x1b[m from \x1b[36m{}:{}\x1b[m to \x1b[36m{}:{}\x1b[m", from.line, from.character, to.line, to.character);

                // Remove \n after block end
                if iter.peek().is_some_and(|&token| matches!(token.kind, NewLine)) {
                    trace!("\x1b[31m-\x1b[m {}", iter.next().unwrap());
                }

                continue;
            }
            // String
            (Quote, _) => {
                let mut new = Token::new(Symbol("".into()), token.position);

                trace!("\x1b[32m+\x1b[m {token}");
                array.push(token.clone());

                while let Some(token) = iter.next() {
                    match token.kind {
                        Quote => {
                            new.position += token.position;

                            trace!("\x1b[32m+\x1b[m {new}");
                            trace!("\x1b[32m+\x1b[m {token}");
                            array.push(new);
                            array.push(token.clone());

                            break;
                        }
                        _ => new.join(token).unwrap(),
                    }
                }

                continue;
            }
            // Ignore whitespaces
            (Space | Tab, _) => {
                trace!("\x1b[31m-\x1b[m {token}");
                continue;
            }
            _ => {}
        }

        trace!("\x1b[32m+\x1b[m {token}");
        array.push(token.clone());
    }

    debug!("\x1b[1;33m*\x1b[37m Filtered: \x1b[31m{}\x1b[m", tokens.len() - array.len());
    debug!("\x1b[1;33m*\x1b[37m Remaining: \x1b[32m{}\x1b[m", array.len());

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
            Token::new(NewLine, (1, 38, 1)),
            Token::new(Colon, (2, 0, 1)),
            Token::new(Slash, (2, 1, 1)),
            Token::new(Quote, (2, 2, 1)),
            Token::new(Space, (2, 3, 1)),
            Token::new(Asterisk, (2, 4, 1)),
            Token::new(Equals, (2, 5, 1)),
            Token::new(OpenBracket, (2, 6, 1)),
            Token::new(CloseBracket, (2, 7, 1)),
            Token::new(OpenCurly, (2, 8, 1)),
            Token::new(CloseCurly, (2, 9, 1)),
        ];

        let tokens = tokenize(buffer);

        assert_eq!(tokens, expect);
    }

    #[test]
    fn test_filter() {
        use TokenKind::*;

        let tokens: [Token; 15] = [
            Slash,
            Slash,
            Symbol("comment".into()),
            NewLine,
            Symbol("var".into()),
            Equals,
            Symbol("null".into()),
            NewLine,
            Slash,
            Asterisk,
            Symbol("comment".into()),
            Space,
            Symbol("block".into()),
            Asterisk,
            Slash,
        ].map(|kind| Token::new(kind, (0, 0, 0)));

        let expect: &[Token] = &tokens[4..8];

        let lexed = filter(&tokens);

        assert_eq!(lexed, expect);
    }
}