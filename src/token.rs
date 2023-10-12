#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Token {
    pub kind: TokenKind,
    pub position: TokenPosition,
}

impl Token {
    pub fn new(kind: TokenKind, position: impl Into<TokenPosition>) -> Self {
        Self {
            kind,
            position: position.into(),
        }
    }

    pub fn join(&mut self, rhs: &Token) -> Result<(), &'static str> {
        if let TokenKind::Symbol(ref mut symbol) = self.kind {
            symbol.push_str(rhs.kind.as_str());
            self.position += rhs.position;
        } else {
            return Err("token is not a symbol");
        }

        Ok(())
    }
}

impl std::fmt::Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "\x1b[33m{:?}\x1b[m {}", self.kind, self.position)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TokenKind {
    Symbol(String),
    Dot,
    Comma,
    Colon,
    Quote,
    Slash,
    Asterisk,
    Space,
    Tab,
    Equals,
    NewLine,
    OpenBracket,
    CloseBracket,
    OpenCurly,
    CloseCurly,
}

impl TokenKind {
    pub fn as_str(&self) -> &str {
        use TokenKind::*;

        match self {
            Dot => ".",
            Comma => ",",
            Colon => ":",
            Quote => "\'",
            Slash => "/",
            Asterisk => "*",
            Space => " ",
            Tab => "\t",
            Equals => "=",
            NewLine => "\n",
            OpenBracket => "[",
            CloseBracket => "]",
            OpenCurly => "{",
            CloseCurly => "}",
            Symbol(s) => s.as_str(),
        }
    }
}

impl From<char> for TokenKind {
    fn from(value: char) -> Self {
        use TokenKind::*;

        match value {
            '.' => Dot,
            ',' => Comma,
            ':' => Colon,
            '\'' => Quote,
            '/' => Slash,
            '*' => Asterisk,
            ' ' => Space,
            '\t' => Tab,
            '=' => Equals,
            '\n' => NewLine,
            '[' => OpenBracket,
            ']' => CloseBracket,
            '{' => OpenCurly,
            '}' => CloseCurly,
            s => Symbol(s.into()),
        }
    }
}

impl Default for TokenKind {
    fn default() -> Self {
        Self::Symbol("".into())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TokenPosition {
    pub line: isize,
    pub character: isize,
    pub length: isize,
}

impl From<(isize, isize, isize)> for TokenPosition {
    fn from(value: (isize, isize, isize)) -> Self {
        Self {
            line: value.0,
            character: value.1,
            length: value.2,
        }
    }
}

impl std::fmt::Display for TokenPosition {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.length > 1 {
            write!(f, "from \x1b[36m{}:{}\x1b[m to \x1b[36m{}:{}\x1b[m", self.line, self.character, self.line, self.character + self.length)
        } else {
            write!(f, "at \x1b[36m{}:{}\x1b[m", self.line, self.character)
        }
    }
}

// impl std::ops::Add for TokenPosition {
//     type Output = Self;

//     fn add(self, rhs: Self) -> Self::Output {
//         Self {
//             length: self.length + rhs.length,
//             ..self
//         }
//     }
// }

impl Default for TokenPosition {
    fn default() -> Self {
        Self {
            line: 1,
            character: 0,
            length: 0,
        }
    }
}

impl std::ops::AddAssign for TokenPosition {
    fn add_assign(&mut self, rhs: Self) {
        self.length += rhs.length;
    }
}
