//! The AST.
//!
//! TODO: error handling

use std::collections::VecDeque;

use crate::error::{Error, Kind, Result};
use crate::tokenizer::{Op, Token, TokenKind, TokenStream};
use crate::{Loc, Span};

trait Ast {
    fn span(&self) -> Span;
    fn parse(tokens: &mut TokenStream) -> Result<Self>
    where
        Self: Sized;
    // fn run(&self);
}

struct Blck {
    stmts: VecDeque<Stmt>,
}

impl Ast for Blck {
    fn span(&self) -> Span {
        Span {
            start: self
                .stmts
                .front()
                .map_or(Loc { row: 1, col: 1 }, |s| s.span().start),
            end: self
                .stmts
                .back()
                .map_or(Loc { row: 1, col: 1 }, |s| s.span().end),
        }
    }

    fn parse(tokens: &mut TokenStream) -> Result<Self> {
        let mut stmts = VecDeque::new();
        while tokens.current().is_some() {
            stmts.push_back(Stmt::parse(tokens)?);
        }

        Ok(Self { stmts })
    }
}

#[derive(PartialEq, Eq, Debug)]
struct Stmt {
    data: StmtData,
    span: Span,
}

impl Stmt {
    fn parse_asgn(tokens: &mut TokenStream) -> Result<Self> {
        let start = tokens.current().unwrap().span.start;
        let name = match tokens.take().kind {
            TokenKind::Ident(ident) => ident,
            _ => {
                return Err(Error {
                    kind: Kind::Parser,
                    span: tokens.current().unwrap().span,
                });
            }
        };

        tokens.eat(&TokenKind::Op(Op::Eq));
        let expn = Expn::parse(tokens)?;

        Ok(Self {
            span: Span {
                start,
                end: expn.span().end,
            },
            data: StmtData::Asgn(name, expn),
        })
    }

    fn parse_prnt(tokens: &mut TokenStream) -> Result<Self> {
        let start = tokens.current().unwrap().span.start;
        tokens.eat(&TokenKind::Ident("print".to_string()));
        tokens.eat(&TokenKind::LParen);
        let expn = Expn::parse(tokens)?;
        let end = tokens.current().unwrap().span.end;
        tokens.eat(&TokenKind::RParen);
        Ok(Self {
            span: Span { start, end },
            data: StmtData::Prnt(expn),
        })
    }
}

impl Ast for Stmt {
    fn span(&self) -> Span {
        self.span
    }

    fn parse(tokens: &mut TokenStream) -> Result<Self> {
        Ok(
            match tokens
                .current()
                .expect("parsing a statement; have already checked not none")
            {
                Token {
                    span,
                    kind: TokenKind::Ident(ident),
                } if ident.as_str() == "pass" => Self {
                    span: *span,
                    data: StmtData::Pass,
                },
                Token {
                    kind: TokenKind::Ident(ident),
                    ..
                } if ident.as_str() == "print" => Self::parse_prnt(tokens)?,
                _ => Self::parse_asgn(tokens)?,
            },
        )
    }
}

#[derive(PartialEq, Eq, Debug)]
enum StmtData {
    Asgn(String, Expn),
    Pass,
    Prnt(Expn),
}

#[derive(PartialEq, Eq, Debug)]
enum Expn {
    BinOp {
        left: Box<Self>,
        right: Box<Self>,
        op: BinOp,
    },
    Leaf(Leaf),
}

impl Expn {
    fn parse_impl(tokens: &mut TokenStream, min_bp: u8) -> Result<Self> {
        let tkn = tokens.current_or()?;
        let span = tkn.span;
        let mut lhs = match tkn.kind {
            TokenKind::LParen => {
                tokens.advance();
                let lhs = Self::parse_impl(tokens, 0)?;
                tokens.eat(&TokenKind::RParen);
                lhs
            }
            TokenKind::Ident(_) | TokenKind::Number(_) => Self::Leaf(Leaf::parse(tokens)?),
            _ => {
                return Err(Error {
                    span,
                    kind: Kind::Parser,
                });
            }
        };

        while let Some(tkn) = tokens.current() {
            let span = tkn.span;
            let op = match tkn.kind {
                TokenKind::NewLine => {
                    tokens.advance();
                    break;
                }
                TokenKind::RParen => {
                    break;
                }
                TokenKind::Op(op) => BinOp::from(op),
                _ => {
                    return Err(Error {
                        span,
                        kind: Kind::Parser,
                    });
                }
            };

            let (l_bp, _) = op.bp();
            if l_bp < min_bp {
                // done with this parse tree
                break;
            }

            tokens.advance();
            let rhs = Self::parse_impl(tokens, l_bp)?;
            lhs = Self::BinOp {
                left: Box::new(lhs),
                right: Box::new(rhs),
                op,
            }
        }

        Ok(lhs)
    }
}

impl Ast for Expn {
    fn span(&self) -> Span {
        match self {
            Self::Leaf(leaf) => leaf.span(),
            Self::BinOp { left, right, .. } => Span {
                start: left.span().start,
                end: right.span().end,
            },
        }
    }

    fn parse(tokens: &mut TokenStream) -> Result<Self> {
        Self::parse_impl(tokens, 0)
    }
}

#[derive(PartialEq, Eq, Debug, Copy, Clone)]
enum BinOp {
    Plus,
    Minus,
    Times,
    Div,
    Mod,
    Expt,
}

impl From<Op> for BinOp {
    fn from(op: Op) -> Self {
        match op {
            Op::Plus => Self::Plus,
            Op::Minus => Self::Minus,
            Op::Times => Self::Times,
            Op::Div => Self::Div,
            Op::Mod => Self::Mod,
            Op::Eq => panic!(),
        }
    }
}

impl BinOp {
    const fn bp(self) -> (u8, u8) {
        match self {
            Self::Plus | Self::Minus => (1, 2),
            Self::Times | Self::Div | Self::Mod => (3, 4),
            Self::Expt => (5, 6),
        }
    }
}

#[derive(PartialEq, Eq, Debug)]
struct Leaf {
    data: LeafData,
    span: Span,
}

impl Leaf {
    fn parse_inpt(_tokens: &mut TokenStream) -> Result<Self> {
        todo!();
    }
}

impl Ast for Leaf {
    fn span(&self) -> Span {
        self.span
    }

    fn parse(tokens: &mut TokenStream) -> Result<Self> {
        let tkn = tokens.take();
        let span = tkn.span;
        Ok(match tkn.kind {
            TokenKind::Ident(s) if s == "input" => Self::parse_inpt(tokens)?,
            TokenKind::Ident(s) => Self {
                data: LeafData::Name(s),
                span,
            },
            TokenKind::Number(n) => Self {
                data: LeafData::Nmbr(n),
                span,
            },
            _ => panic!(),
        })
    }
}

#[derive(PartialEq, Eq, Debug)]
enum LeafData {
    Name(String),
    Nmbr(u32),
    Inpt(String),
}

struct Prgm {
    main: Blck,
}

impl Ast for Prgm {
    fn span(&self) -> Span {
        self.main.span()
    }

    fn parse(tokens: &mut TokenStream) -> Result<Self> {
        Ok(Self {
            main: Blck::parse(tokens)?,
        })
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    mod expn {
        use super::*;
        use crate::tokenizer::Tokenizer;

        // macro_rules! expn_test {
        //     ($name:ident: $in:expn => $out:expn ) => ()
        // }

        macro_rules! num {
            ($srow:expr,$scol:expr; $erow:expr,$ecol:expr => $num:expr) => {
                Box::new(Expn::Leaf(Leaf {
                    data: LeafData::Nmbr($num),
                    span: Span {
                        start: Loc {
                            row: $srow,
                            col: $scol,
                        },
                        end: Loc {
                            row: $erow,
                            col: $ecol,
                        },
                    },
                }
            ))};
            ($srow:expr,$scol:expr => $num:expr) => {num!($srow,$scol;$srow,$scol => $num)}
        }

        #[test]
        fn two_plus_two() {
            let mut tokens = Tokenizer::lex("2+2").unwrap();
            let expn = Expn::parse(&mut tokens).unwrap();

            assert_eq!(expn, Expn::BinOp {
                left: num!(1,1 => 2),
                op: BinOp::Plus,
                right: num!(1,3 => 2),
            });
        }

        #[test]
        fn two_plus_two_times_four() {
            let mut tokens = Tokenizer::lex("2+2*4").unwrap();
            let expn = Expn::parse(&mut tokens).unwrap();

            assert_eq!(expn, Expn::BinOp {
                left: num!(1,1 => 2),
                op: BinOp::Plus,
                right: Box::new(Expn::BinOp {
                    left: num!(1,3 => 2),
                    op: BinOp::Times,
                    right: num!(1,5 => 4),
                })
            });
        }

        #[test]
        fn paren_two_plus_two_times_four() {
            let mut tokens = Tokenizer::lex("(2+2)*4").unwrap();
            let expn = Expn::parse(&mut tokens).unwrap();

            assert_eq!(expn, Expn::BinOp {
                left: Box::new(Expn::BinOp {
                    left: num!(1,2 => 2),
                    op: BinOp::Plus,
                    right: num!(1,4 => 2),
                }),
                op: BinOp::Times,
                right: num!(1,7 => 4),
            });
        }

        #[test]
        fn assgn() {
            let mut tokens = Tokenizer::lex("x=2+2").unwrap();
            let stmt = Stmt::parse(&mut tokens).unwrap();

            assert_eq!(stmt, Stmt {
                span: Span {
                    start: Loc {
                        row: 1,
                        col: 1,
                    },
                    end: Loc {
                        row: 1,
                        col: 5
                    },
                },
                data: StmtData::Asgn(
                    "x".to_string(),
                    Expn::BinOp {
                        left: num!(1,3 => 2),
                        op: BinOp::Plus,
                        right: num!(1,5 => 2)
                    }
                ),
            });
        }
    }
}
