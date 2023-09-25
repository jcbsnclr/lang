mod eval;
mod lexer;
mod parser;
mod string;
mod tree;

use std::{fs, io};

use lexer::Lexer;

fn main() -> anyhow::Result<()> {
    let source = fs::read_to_string("example/basic")?;

    eval::eval(&source);

    Ok(())
}
