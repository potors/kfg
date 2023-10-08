pub(crate) mod token;
pub use token::*;

pub(crate) mod ast;
pub use ast::*;

pub(crate) mod lexer;
pub(crate) mod parser;
pub use parser::ParserError;

pub struct Kfg;

impl Kfg {
    pub fn read(value: &str) -> Result<Ast, parser::ParserError> {
        let content = std::fs::read(value).expect("couldn't read the file");

        Self::parse(&content)
    }

    pub fn parse(buffer: &[u8]) -> Result<Ast, parser::ParserError> {
        let tokens = lexer::lex(buffer);

        parser::parse(&tokens)
    }
}
