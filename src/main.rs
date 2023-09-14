mod lexer;
mod parser;

use std::{fs, io};

use lexer::Lexer;

fn main() -> io::Result<()> {
    let source = fs::read_to_string("example/test")?;

    dbg!(parser::parse(&source));

    Ok(())
}
