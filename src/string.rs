use crate::lexer::{Span, Token, TokenKind};
use crate::parser::{ParserError, ParserErrorKind};

use std::iter::Peekable;
use std::str::CharIndices;

pub fn parse_string(token: Token) -> Result<String, ParserError> {
    assert_eq!(token.kind(), TokenKind::String);

    if token.view().chars().last() != Some('"') {
        return Err(ParserError {
            span: token.span(),
            kind: ParserErrorKind::UnclosedString,
        });
    }

    let mut string = String::new();

    let mut stream = token.view().char_indices();
    let (start, _) = stream.next().unwrap();

    while let Some((pos, c)) = stream.next() {
        if c == '\\' {
            let Some((pos, c)) = stream.next() else {
                return Err(ParserError {
                    span: Span::starting_at(pos),
                    kind: ParserErrorKind::UnclosedString,
                });
            };

            match c {
                'n' => string.push('\n'),
                'r' => string.push('\r'),
                't' => string.push('\t'),
                '"' => string.push('"'),

                _ => {
                    return Err(ParserError {
                        span: Span::starting_at(pos),
                        kind: ParserErrorKind::InvalidEscapeSequence,
                    })
                }
            }
        } else {
            string.push(c);
        }
    }

    string.pop();

    Ok(string)
}
