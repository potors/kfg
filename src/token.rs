#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Token {
    pub kind: TokenKind,
    pub position: TokenPosition,
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

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct TokenPosition {
    pub line: isize,
    pub character: isize,
    pub length: isize,
}

impl ToString for TokenPosition {
    fn to_string(&self) -> String {
        format!("{}:{}-{}", self.line, self.character, self.length)
    }
}
