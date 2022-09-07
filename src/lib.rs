#![allow(dead_code)]

mod error;
mod tokenizer;
mod ast;

pub use error::{Error, Result};

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
