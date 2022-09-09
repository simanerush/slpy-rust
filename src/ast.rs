//! The AST.

use std::collections::VecDeque;

use crate::error::{Error, Kind, Result};
use crate::{Loc, Span};
use crate::tokenizer::TokenStream;

trait Ast {
    fn span(&self) -> Span;
    // fn parse(tokens: &mut TokenStream) -> Self;
    // fn run(&self);
}

struct Blck {
    stmts: VecDeque<Stmt>,
}

impl Ast for Blck {
    fn span(&self) -> Span {
       Span { 
        start: (self.stmts.front().map_or(Loc { row: 1, col: 1 }, Ast::span)), 
        end: (self.stmts.back().map_or(Loc { row: 1, col: 1 }, Ast::span)) 
        }
    }
}

struct Stmt {
   data: StmtData,
   span: Span
} 

impl Ast for Stmt {
    fn span(&self) -> Span {
        self.span
    }
}

enum StmtData {
    Asgn(String, Expn),
    Pass,
    Prnt(Expn),
}

enum Expn {
    BinOp{left: Box<Expn>, right: Box<Expn>, op: BinOp},
    Leaf(Leaf),
}

impl Ast for Expn {
    fn span(&self) -> Span {
        match self {
            Expn::Leaf(leaf) => leaf.span(),
            Expn::BinOp{
                left,
                right,
                ..
            } => Span {
                start: left.span().start,
                end: right.span().end
            }
        }
    }
}

enum BinOp {
    Plus, 
    Minus,
    Times,
    Div,
    Mod,
    Expt,
}

struct Leaf {
    data: LeafData,
    span: Span,
}

impl Ast for Leaf {
    fn span(&self) -> Span {
        self.span
    }
}

enum LeafData {
    Name(String),
    Nmbr(u32),
    Impt(String),
}

struct Prgm {
    main: Blck,
}

impl Ast for Prgm {
    fn span(&self) -> Span {
        return self.main.span();
    }
}