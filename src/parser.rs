use crate::lexer::{BracketKind, Lexer, Span, Token, TokenKind};

type Result<T> = std::result::Result<T, ParserError>;

#[derive(thiserror::Error, Debug, Clone)]
pub enum ParserErrorKind {
    #[error("foo")]
    Foo,
}

#[derive(Debug, Clone)]
pub struct ParserError {
    kind: ParserErrorKind,
    line: usize,
    col: usize,
}

impl<'a> Lexer<'a> {
    fn resolve_line_col(&self, span: Span) -> (usize, usize) {
        let (mut line, mut col) = (1, 1);

        let iter = self
            .source()
            .char_indices()
            .filter_map(|(index, c)| (index < span.start()).then_some(c));

        for c in iter {
            match c {
                '\n' => {
                    line += 1;
                    col = 1;
                }

                _ => col += 1,
            }
        }

        (line, col)
    }
}

#[derive(Debug, Clone)]
pub struct Command {
    data: Vec<Atom>,
    span: Span,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BatchKind {
    Quote,
    Inline,
}

#[derive(Debug, Clone)]
pub enum AtomData {
    Identifier(String),
    Number(u64),

    Batch(BatchKind, Vec<Command>),
}

#[derive(Debug, Clone)]
pub struct Atom {
    data: AtomData,
    span: Span,
}

fn parse_atom(lexer: &mut Lexer) -> Result<Atom> {
    let token = lexer.next().unwrap();

    match token.kind() {
        TokenKind::Identifier => Ok(Atom {
            data: AtomData::Identifier(token.view().to_string()),
            span: token.span(),
        }),

        _ => unimplemented!(),
    }
}

pub fn parse(source: &str) -> Result<Atom> {
    let mut lexer = Lexer::from_source(source);
    parse_atom(&mut lexer)
}
