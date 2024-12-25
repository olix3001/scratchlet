use crate::frontend::parser::lexer::{Token, TokenInfo};

#[derive(Debug, Clone)]
pub enum SyntaxError {
    UnexpectedToken {
        expected: Vec<Token>,
        found: TokenInfo,
    },
}
