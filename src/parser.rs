use crate::lexer::{BracketKind, Lexer, Span, Token, TokenKind};
use crate::string::parse_string;
use crate::tree::{GroupKind, TokenTree, TokenTreeData};

type Result<T> = std::result::Result<T, ParserError>;

#[derive(Debug, Clone)]
pub struct Command {
    data: Vec<Atom>,
    span: Span,
}

impl Command {
    pub fn data(&self) -> &Vec<Atom> {
        &self.data
    }

    pub fn span(&self) -> Span {
        self.span
    }
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
    String(String),

    Batch(Vec<Command>),
    Inline(Vec<Command>),
}

#[derive(Debug, Clone)]
pub struct Atom {
    pub data: AtomData,
    span: Span,
}

impl Atom {
    pub fn data(&self) -> &AtomData {
        &self.data
    }

    pub fn span(&self) -> Span {
        self.span
    }
}

#[derive(thiserror::Error, Debug)]
pub enum ParserErrorKind {
    #[error("unexpected delimiter '{0:?}'")]
    UnexpectedDelimiter(BracketKind),
    #[error("expected delimiter '{0:?}")]
    ExpectedDelimiter(BracketKind),
    #[error("unclosed string literal")]
    UnclosedString,
    #[error("invalid escape sequence")]
    InvalidEscapeSequence,
}

#[derive(thiserror::Error, Debug)]
#[error("{:?}: {kind}")]
pub struct ParserError {
    pub span: Span,
    pub kind: ParserErrorKind,
}

pub fn parse_group(tree: &TokenTree) -> Result<Atom> {
    let span = tree.span();
    let TokenTreeData::Group(group, body) = tree.data() else {
        unreachable!()
    };

    let mut group_body = vec![];
    let atom_data = match group {
        GroupKind::TopLevel | GroupKind::Batch => AtomData::Batch,
        GroupKind::Inline => AtomData::Inline,
    };

    let mut command = vec![];
    let mut command_span = Span::starting_at(tree.span().start() + 1);

    for tree in body.iter() {
        command_span = command_span.join(tree.span());

        match tree.data() {
            TokenTreeData::Token(Token {
                kind: TokenKind::Semicolon,
                ..
            }) => {
                group_body.push(Command {
                    data: command.clone(),
                    span: command_span,
                });

                command.clear();
                command_span = Span::starting_at(command_span.end());
            }

            _ => command.push(parse_atom(tree)?),
        }
    }

    group_body.push(Command {
        data: command,
        span: command_span,
    });

    Ok(Atom {
        data: atom_data(group_body),
        span,
    })
}

pub fn parse_atom(tree: &TokenTree) -> Result<Atom> {
    match tree.data() {
        TokenTreeData::Token(token) => match token.kind() {
            TokenKind::Identifier => Ok(Atom {
                data: AtomData::Identifier(token.view().to_string()),
                span: token.span(),
            }),

            TokenKind::Integer => Ok(Atom {
                data: AtomData::Number(token.view().parse().unwrap()),
                span: token.span(),
            }),

            TokenKind::String => Ok(Atom {
                data: AtomData::String(parse_string(*token)?),
                span: token.span(),
            }),

            _ => unimplemented!("{:?}", token),
        },

        TokenTreeData::Group(..) => parse_group(tree),
    }
}

pub fn parse(source: &str) -> Result<Atom> {
    let tree = TokenTree::from_source(source)?;

    println!("{}", crate::tree::TreeDisplay(0, &tree));

    parse_group(&tree)
}
