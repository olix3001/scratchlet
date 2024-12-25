use logos::{Lexer, Logos};

use crate::common::{
    error::SyntaxError,
    location::{Loc, SourceId},
};

#[derive(Logos, Debug, Clone, Copy, PartialEq)]
#[logos(skip r"//[^\n]*")]
#[logos(skip r"[ \t\r\f]+")] // Do not skip newline as it acts as semicolon
#[rustfmt::skip]
pub enum Token {
    // ==< Operators and separators >==
    #[token("(")] LeftParen,
    #[token(")")] RightParen,
    #[token("[")] LeftBracket,
    #[token("]")] RightBracket,
    #[token("{")] LeftCurly,
    #[token("}")] RightCurly,
    #[token("<")] LeftAngle,
    #[token(">")] RightAngle,

    #[token(":")] Colon,
    #[token("::")] DoubleColon,
    #[token(".")] Dot,
    #[token("->")] Arrow,

    // ==< Symbols >==
    #[token("@")] AtSymbol,

    // ==< Keywords >==
    #[token("sprite")] KwSprite,
    #[token("import")] KwImport,
    #[token("proc")] KwProc,

    // ==< Literals >==
    #[regex(r#""([^"\\]|\\t|\\u|\\n|\\")*""#)] StringLiteral,
    #[regex(r"[0-9][0-9_]*\.[0-9][0-9_]*")] NumberLiteral,

    // ==< Other >==
    #[token("\n")] NL,
    #[regex(r"[_\p{L}][_\p{L}\p{N}]*")] Identifier,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TokenInfo(Token, String, Loc);

#[rustfmt::skip]
impl TokenInfo {
    #[inline(always)]
    pub fn kind(&self) -> Token { self.0 }
    #[inline(always)]
    pub fn text(&self) -> String { self.1.clone() }
    #[inline(always)]
    pub fn loc(&self) -> Loc { self.2.clone() }
}

#[derive(Debug, Clone)]
pub struct Tokens<'src> {
    iter: Lexer<'src, Token>,
    stack: Vec<TokenInfo>,
    current: usize,
    pub(crate) source: SourceId,
    span_stack: Vec<usize>,
}

impl<'src> Tokens<'src> {
    pub fn from_lexer(iter: Lexer<'src, Token>, source: SourceId) -> Self {
        Self {
            iter,
            stack: Vec::new(),
            current: 0,
            source,

            span_stack: Vec::new(),
        }
    }

    pub fn from_string(text: &'src str, source: SourceId) -> Self {
        Self::from_lexer(Token::lexer(text), source)
    }

    pub fn next(&mut self) -> Option<Token> {
        if self.current == self.stack.len() {
            let Some(Ok(next_elem)) = self.iter.next() else {
                return None;
            };

            let slice = self.iter.slice().to_string();
            let range = self.iter.span();

            self.stack
                .push(TokenInfo(next_elem, slice, Loc::new(range, self.source)));
            self.current += 1;
            Some(next_elem)
        } else {
            if self.current >= self.stack.len() {
                return None;
            }
            self.current += 1;
            Some(self.stack[self.current - 1].0)
        }
    }

    pub fn next_info(&mut self) -> Option<TokenInfo> {
        if self.next().is_some() {
            self.current_info()
        } else {
            None
        }
    }

    pub fn current(&self) -> Option<Token> {
        if self.current < 1 || self.current - 1 > self.stack.len() {
            return None;
        }
        Some(self.stack[self.current - 1].0)
    }

    pub fn current_info(&self) -> Option<TokenInfo> {
        if self.current < 1 || self.current - 1 > self.stack.len() {
            return None;
        }
        Some(self.stack[self.current - 1].clone())
    }

    pub fn back(&mut self) -> bool {
        if self.current < 1 {
            return false;
        }
        self.current -= 1;
        true
    }

    pub fn peek(&mut self) -> Option<Token> {
        let next = self.next();
        if next.is_some() {
            self.current -= 1;
        }
        next
    }

    pub fn peek_info(&mut self) -> Option<TokenInfo> {
        let next = self.next();
        let info = self.current_info();
        if next.is_some() {
            self.current -= 1;
        }
        info
    }

    pub fn is(&mut self, token: Token) -> bool {
        let next = self.next();
        if next == Some(token) {
            true
        } else if next == None {
            false
        } else {
            self.current -= 1;
            false
        }
    }

    pub fn is_any(&mut self, tokens: &[Token]) -> Option<Token> {
        let next = self.next();
        if let Some(next) = next {
            if tokens.contains(&next) {
                return Some(next);
            }
            self.back();
        }
        None
    }

    pub fn loc(&self) -> Option<Loc> {
        if self.current < 1 || self.current - 1 > self.stack.len() {
            return None;
        }
        Some(self.stack[self.current - 1].2.clone())
    }

    pub fn text(&self) -> Option<&str> {
        if self.current < 1 || self.current - 1 > self.stack.len() {
            return None;
        }
        Some(self.stack[self.current - 1].1.as_str())
    }

    // ==< Parser utilities >==
    pub fn expect(&mut self, token: Token) -> Result<TokenInfo, SyntaxError> {
        if self.next() != Some(token) {
            Err(SyntaxError::UnexpectedToken {
                expected: vec![token],
                found: self.current_info().unwrap(),
            })
        } else {
            Ok(self.current_info().unwrap())
        }
    }

    pub fn begin_span(&mut self) {
        let start = self.peek_info().unwrap().loc().span.start;
        self.span_stack.push(start);
    }

    pub fn end_span(&mut self) -> Loc {
        Loc::new(
            self.span_stack.pop().unwrap()..self.loc().unwrap().span.end,
            self.source,
        )
    }
}
