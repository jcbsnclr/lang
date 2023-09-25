use crate::lexer::{BracketKind, Lexer, Span, Token, TokenKind};
use crate::parser::{ParserError, ParserErrorKind};

use std::fmt::{self, Display};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GroupKind {
    Batch,
    Inline,
    TopLevel,
}

impl From<BracketKind> for GroupKind {
    fn from(bracket: BracketKind) -> GroupKind {
        match bracket {
            BracketKind::Curly => GroupKind::Batch,
            BracketKind::Square => GroupKind::Inline,

            BracketKind::Paren => todo!("token-level quoting"),
        }
    }
}

impl Into<BracketKind> for GroupKind {
    fn into(self) -> BracketKind {
        match self {
            GroupKind::Batch => BracketKind::Curly,
            GroupKind::Inline => BracketKind::Square,

            GroupKind::TopLevel => unreachable!(),
        }
    }
}

#[derive(Debug, Clone)]
pub enum TokenTreeData<'a> {
    Group(GroupKind, Vec<TokenTree<'a>>),

    Token(Token<'a>),
}

/// A structured sequence of tokens
#[derive(Debug, Clone)]
pub struct TokenTree<'a> {
    span: Span,
    data: TokenTreeData<'a>,
}

impl<'a> TokenTree<'a> {
    pub fn from_source(source: &'a str) -> Result<TokenTree<'a>, ParserError> {
        let mut lexer = Lexer::from_source(source);
        let mut stack = Stack::new();

        stack.feed(&mut lexer)?;

        stack.finalise()
    }

    pub fn span(&self) -> Span {
        self.span
    }

    pub fn data(&self) -> &TokenTreeData<'a> {
        &self.data
    }
}

struct Stack<'a> {
    stack: Vec<(Span, GroupKind, Vec<TokenTree<'a>>)>,
}

impl<'a> Stack<'a> {
    fn new() -> Stack<'a> {
        Stack {
            stack: vec![(Span::starting_at(0), GroupKind::TopLevel, vec![])],
        }
    }

    fn push(&mut self, tree: TokenTree<'a>) {
        let (_, _, body) = self
            .stack
            .iter_mut()
            .last()
            .expect("token tree stack exhausted");

        body.push(tree);
    }

    fn open(&mut self, start: Span, kind: BracketKind) {
        self.stack.push((start, kind.into(), vec![]));
    }

    fn close(&mut self, end: Span, kind: BracketKind) -> Result<(), ParserError> {
        let (start, expected_group, body) = self.stack.pop().expect("token tree stack exhausted");

        let found_group: GroupKind = kind.into();

        if found_group != expected_group {
            return Err(ParserError {
                span: start.join(end),
                kind: ParserErrorKind::UnexpectedDelimiter(kind),
            });
        }

        let tree = TokenTree {
            span: start.join(end),
            data: TokenTreeData::Group(expected_group, body),
        };

        self.push(tree);

        Ok(())
    }

    fn finalise(mut self) -> Result<TokenTree<'a>, ParserError> {
        let (span, group, body) = self.stack.pop().expect("token tree stack exhausted");

        if self.stack.len() != 0 {
            return Err(ParserError {
                span,
                kind: ParserErrorKind::ExpectedDelimiter(group.into()),
            });
        } else {
            Ok(TokenTree {
                span,
                data: TokenTreeData::Group(group, body),
            })
        }
    }

    fn feed(&mut self, lexer: &mut Lexer<'a>) -> Result<(), ParserError> {
        for token in lexer {
            self.stack[0].0 = self.stack[0].0.join(token.span());

            match token.kind() {
                TokenKind::Whitespace | TokenKind::Comment => continue,

                TokenKind::Open(bracket) => self.open(token.span(), bracket),
                TokenKind::Close(bracket) => self.close(token.span(), bracket)?,

                _ => self.push(TokenTree {
                    span: token.span(),
                    data: TokenTreeData::Token(token),
                }),
            }
        }

        Ok(())
    }
}

pub struct TreeDisplay<'a>(pub usize, pub &'a TokenTree<'a>);

impl<'a> Display for TreeDisplay<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for _ in 0..self.0 {
            write!(f, " ")?;
        }

        match &self.1.data {
            TokenTreeData::Token(t) => {
                write!(f, "{:?} ", t.kind())?;

                if matches!(t.kind(), TokenKind::Integer | TokenKind::Identifier) {
                    write!(f, "{:?}", t.view())?;
                }

                writeln!(f)?;
            }

            TokenTreeData::Group(kind, body) => {
                writeln!(f, "{:?}:", kind)?;

                for tree in body.iter() {
                    write!(f, "{}", TreeDisplay(self.0 + 2, tree))?;
                }
            }
        }

        Ok(())
    }
}
