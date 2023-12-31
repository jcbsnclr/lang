use std::iter::Peekable;
use std::ops::Range;
use std::str::CharIndices;

/// A [Span] represents a slice of the input source string
#[derive(Debug, Copy, Clone)]
pub struct Span {
    start: usize,
    end: usize,
}

impl Span {
    pub fn new(start: usize, end: usize) -> Span {
        Span { start, end }
    }

    pub fn starting_at(start: usize) -> Span {
        Span::new(start, start + 1)
    }

    pub fn extend_to(&mut self, pos: usize) {
        self.end = pos + 1;
    }

    pub fn start(&self) -> usize {
        self.start
    }

    pub fn end(&self) -> usize {
        self.end
    }

    pub fn as_range(&self) -> Range<usize> {
        self.start..self.end
    }

    pub fn join(self, rhs: Span) -> Span {
        Span::new(self.start, rhs.end)
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum BracketKind {
    Paren,
    Square,
    Curly,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum TokenKind {
    Whitespace,
    Comment,

    Open(BracketKind),
    Close(BracketKind),
    Semicolon,

    Identifier,
    Integer,
    String,

    Unknown(char),
}

#[derive(Debug, Copy, Clone)]
pub struct Token<'a> {
    pub kind: TokenKind,
    pub span: Span,
    pub view: &'a str,
}

impl<'a> Token<'a> {
    pub fn kind(&self) -> TokenKind {
        self.kind
    }

    pub fn span(&self) -> Span {
        self.span
    }

    pub fn view(&self) -> &'a str {
        self.view
    }
}

pub struct Lexer<'a> {
    source: &'a str,
    stream: Peekable<CharIndices<'a>>,
    peek: Option<Token<'a>>,
}

impl<'a> Lexer<'a> {
    pub fn from_source(source: &'a str) -> Lexer<'a> {
        let mut lexer = Lexer {
            source,
            stream: source.char_indices().peekable(),
            peek: None,
        };

        lexer.next();

        lexer
    }

    pub fn peek(&self) -> Option<Token<'a>> {
        self.peek
    }

    pub fn source(&self) -> &'a str {
        self.source
    }

    fn take_while(&mut self, span: &mut Span, cond: impl Fn(char) -> bool) {
        while let Some((pos, _)) = self.stream.next_if(|&(_, c)| cond(c)) {
            span.extend_to(pos);
        }
    }

    fn produce_token(&self, kind: TokenKind, span: Span) -> Token<'a> {
        Token {
            kind,
            span,
            view: &self.source[span.as_range()],
        }
    }

    fn produce_while(&mut self, kind: TokenKind, cond: impl Fn(char) -> bool) -> Option<Token<'a>> {
        let &(start, _) = self.stream.peek()?;
        let mut span = Span::starting_at(start);

        self.take_while(&mut span, cond);

        Some(self.produce_token(kind, span))
    }

    pub fn resolve_line_col(&self, span: Span) -> (usize, usize) {
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

impl<'a> Iterator for Lexer<'a> {
    type Item = Token<'a>;

    fn next(&mut self) -> Option<Token<'a>> {
        let &(_, c) = self.stream.peek()?;

        let result = self.peek;

        self.peek = match c {
            c if Lexer::ident_start(c) => {
                self.produce_while(TokenKind::Identifier, Lexer::ident_body)
            }
            c if c.is_whitespace() => {
                self.produce_while(TokenKind::Whitespace, char::is_whitespace)
            }

            '0'..='9' => self.produce_while(TokenKind::Integer, |c| matches!(c, '0'..='9')),

            '#' => self.produce_while(TokenKind::Comment, |c| c != '\n'),

            '"' => self.produce_string(),

            _ => self.produce_punctuation(),
        };

        result
    }
}

impl<'a> Lexer<'a> {
    fn produce_punctuation(&mut self) -> Option<Token<'a>> {
        let (start, c) = self.stream.next()?;
        let span = Span::starting_at(start);

        let kind = match c {
            '(' => TokenKind::Open(BracketKind::Paren),
            ')' => TokenKind::Close(BracketKind::Paren),
            '[' => TokenKind::Open(BracketKind::Square),
            ']' => TokenKind::Close(BracketKind::Square),
            '{' => TokenKind::Open(BracketKind::Curly),
            '}' => TokenKind::Close(BracketKind::Curly),

            ';' => TokenKind::Semicolon,

            c => TokenKind::Unknown(c),
        };

        Some(self.produce_token(kind, span))
    }

    fn produce_string(&mut self) -> Option<Token<'a>> {
        let (start, _) = self.stream.next()?;
        let mut span = Span::starting_at(start);

        while let Some((pos, c)) = self.stream.next() {
            // extend token to current char
            span.extend_to(pos);

            if c == '\\' {
                // escape sequence; skip next char
                if let Some((pos, _)) = self.stream.next() {
                    // extend token to skipped char
                    span.extend_to(pos);
                }
            } else if c == '"' {
                // end quote
                break;
            }
        }

        // produce string literal token
        Some(self.produce_token(TokenKind::String, span))
    }
}

// lexer rules
impl<'a> Lexer<'a> {
    fn ident_start(c: char) -> bool {
        c.is_alphabetic() || c == '_'
    }

    fn ident_body(c: char) -> bool {
        Lexer::ident_start(c) || c.is_alphanumeric()
    }
}
