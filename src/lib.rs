#![allow(dead_code)]

mod ast;
mod error;
mod repl;
mod tokenizer;

pub use error::{Error, Result};
pub use repl::repl;

use crate::ast::{Ast, Prgm};
use crate::tokenizer::Tokenizer;
use std::fs;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Loc {
    pub row: usize,
    pub col: usize,
}

impl std::fmt::Display for Loc {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "row {}, col {}", self.row, self.col)
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Span {
    pub start: Loc,
    pub end: Loc,
}

/// Dump the AST.
///
/// # Errors
/// If parsing fils.
pub fn dump(source: String) -> Result<()> {
    let contents = fs::read_to_string(source).expect("Should have been able to read the file");
    let mut tokens = Tokenizer::lex(contents.as_str())?;
    let parsed = Prgm::parse(&mut tokens)?;
    print!("{}", parsed.dump(0));

    Ok(())
}

