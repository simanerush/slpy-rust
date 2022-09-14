//! The AST.
//!
//! TODO: error handling

use std::collections::{HashMap, VecDeque};

use crate::error::{Error, Kind, Result};
use crate::tokenizer::{Op, Token, TokenKind, TokenStream};
use crate::{Loc, Span};

type SlpyObject = i32;

#[derive(Default)]
pub struct Context(HashMap<String, SlpyObject>);

impl Context {
    fn get(&self, name: &str) -> Option<SlpyObject> {
        self.0.get(name).copied()
    }

    fn set(&mut self, name: String, val: i32) {
        self.0.insert(name, val);
    }
}

pub trait Ast: Sized {
    type Output;

    fn span(&self) -> Span;

    fn parse(tokens: &mut TokenStream) -> Result<Self>;

    fn eval(self, ctx: &mut Context) -> Result<Self::Output>;

    fn parse_and_eval(mut tokens: TokenStream, ctx: &mut Context) -> Result<Self::Output> {
        Self::parse(&mut tokens)?.eval(ctx)
    }

    fn dump(&self, indent: usize) -> String;
}

struct Blck {
    stmts: VecDeque<Stmt>,
}

impl Ast for Blck {
    type Output = ();

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
            tokens.eat(&TokenKind::NewLine);
        }

        Ok(Self { stmts })
    }

    fn eval(self, ctx: &mut Context) -> Result<Self::Output> {
        for stmt in self.stmts {
            stmt.eval(ctx)?;
        }
        Ok(())
    }

    fn dump(&self, indent: usize) -> String {
        " ".repeat(indent)
            + "Blck\n"
            + &self
                .stmts
                .iter()
                .map(|s| s.dump(indent + 1))
                .fold(String::new(), |s, n| s + &n + "\n")
    }
}

#[derive(PartialEq, Eq, Debug)]
pub struct Stmt {
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

        let tkn = tokens.current_or()?;
        let span = tkn.span;

        let expn = if tkn.kind == TokenKind::Op(Op::AddEq) {
            tokens.advance();
            let constant = Expn::parse(tokens)?;
            Expn::BinOp {
                left: Box::new(Expn::Leaf(Leaf {
                    data: LeafData::Name(name.clone()),
                    span,
                })),
                right: Box::new(constant),
                op: BinOp::Plus
            }
        } else {
            tokens.eat(&TokenKind::Op(Op::Eq));

            Expn::parse(tokens)?
        };
            
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
    type Output = ();

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

    fn eval(self, ctx: &mut Context) -> Result<()> {
        match self.data {
            StmtData::Asgn(name, expn) => {
                let val = expn.eval(ctx)?;
                ctx.set(name, val);
            }
            StmtData::Prnt(e) => {
                println!("{}", e.eval(ctx)?);
            }
            StmtData::Pass => {}
        }

        Ok(())
    }

    fn dump(&self, indent: usize) -> String {
        match &self.data {
            StmtData::Asgn(name, expn) => {
                " ".repeat(indent)
                    + "Asgn\n"
                    + &" ".repeat(indent + 1)
                    + name
                    + "\n"
                    + &expn.dump(indent + 1)
            }
            StmtData::Prnt(expn) => " ".repeat(indent) + "Prnt\n" + &expn.dump(indent + 1),
            StmtData::Pass => " ".repeat(indent) + "Pass",
        }
    }
}

#[derive(PartialEq, Eq, Debug)]
enum StmtData {
    Asgn(String, Expn),
    Pass,
    Prnt(Expn),
}

#[derive(PartialEq, Eq, Debug)]
pub enum Expn {
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
                TokenKind::Op(op) => BinOp::from_token(op, span)?,
                _ => {
                    return Err(Error {
                        span,
                        kind: Kind::Parser,
                    });
                }
            };

            let (l_bp, r_bp) = op.bp();
            if l_bp < min_bp {
                // done with this parse tree
                break;
            }

            tokens.advance();
            let rhs = Self::parse_impl(tokens, r_bp)?;
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
    type Output = SlpyObject;

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

    fn eval(self, ctx: &mut Context) -> Result<Self::Output> {
        match self {
            Self::BinOp { left, right, op } => Ok(op.eval(left.eval(ctx)?, right.eval(ctx)?)),
            Self::Leaf(l) => l.eval(ctx),
        }
    }

    fn dump(&self, indent: usize) -> String {
        match &self {
            Self::BinOp { left, right, op } => {
                " ".repeat(indent)
                    + op.as_str()
                    + "\n"
                    + &left.dump(indent + 1)
                    + "\n"
                    + &right.dump(indent + 1)
            }
            Self::Leaf(l) => l.dump(indent),
        }
    }
}

#[derive(PartialEq, Eq, Debug, Copy, Clone)]
pub enum BinOp {
    Plus,
    Minus,
    Times,
    Div,
    Mod,
    Expt,
}

impl BinOp {
    const fn from_token(op: Op, span: Span) -> Result<Self> {
        Ok(match op {
            Op::Plus => Self::Plus,
            Op::Minus => Self::Minus,
            Op::Times => Self::Times,
            Op::Div => Self::Div,
            Op::Mod => Self::Mod,
            Op::Expt => Self::Expt,
            Op::Eq | Op::AddEq => {
                return Err(Error {
                    kind: Kind::Parser,
                    span,
                });
            }
        })
    }

    const fn bp(self) -> (u8, u8) {
        match self {
            Self::Plus | Self::Minus => (1, 2),
            Self::Times | Self::Div | Self::Mod => (3, 4),
            Self::Expt => (6, 5),
        }
    }

    fn eval(self, lhs: SlpyObject, rhs: SlpyObject) -> SlpyObject {
        match self {
            Self::Plus => lhs + rhs,
            Self::Minus => lhs - rhs,
            Self::Times => lhs * rhs,
            Self::Div => lhs / rhs,
            Self::Mod => lhs % rhs,
            // if rhs is negative, then we have a fractionl result as the output, which we round to
            // zero.
            Self::Expt => u32::try_from(rhs).map_or_else(|_| 0, |n| lhs.pow(n)),
        }
    }

    const fn as_str(self) -> &'static str {
        match self {
            Self::Plus => "Plus",
            Self::Minus => "Mnus",
            Self::Times => "Tmes",
            Self::Div => "IDiv",
            Self::Mod => "Modu",
            Self::Expt => "Expt",
        }
    }
}

#[derive(PartialEq, Eq, Debug)]
pub struct Leaf {
    data: LeafData,
    span: Span,
}

impl Leaf {
    fn parse_inpt(tokens: &mut TokenStream) -> Result<Self> {
        // todo: fix
        let start = tokens.current().unwrap().span.start;
        tokens.eat(&TokenKind::Ident("input".to_string()));
        tokens.eat(&TokenKind::LParen);
        let tkn = tokens.current_or()?;
        let prompt = if let TokenKind::Str(s) = &tkn.kind {
            s.to_string()
        } else {
            return Err(Error {
                span: tkn.span,
                kind: Kind::Parser,
            });
        };
        let end = tokens.current().unwrap().span.end;
        tokens.advance();
        tokens.eat(&TokenKind::RParen);
        Ok(Self {
            span: Span { start, end },
            data: LeafData::Inpt(prompt),
        })
    }
}

impl Ast for Leaf {
    type Output = SlpyObject;

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

    fn eval(self, ctx: &mut Context) -> Result<Self::Output> {
        let err = Error {
            kind: Kind::Interpretation,
            span: self.span,
        };
        Ok(match self.data {
            LeafData::Name(s) => ctx.get(s.as_str()).ok_or(err)?,
            // TODO: handle too-large numbers gracefully
            #[allow(clippy::cast_possible_wrap)]
            LeafData::Nmbr(n) => n as SlpyObject,
            LeafData::Inpt(s) => {
                print!("{}", s);
                let mut buffer = String::new();
                if std::io::stdin().read_line(&mut buffer).is_ok() {
                    if let Ok(n) = buffer.parse() {
                        return Ok(n);
                    }
                }
                return Err(err);
            }
        })
    }

    fn dump(&self, indent: usize) -> String {
        match &self.data {
            LeafData::Name(name) => {
                " ".repeat(indent) + "Lkup\n" + &" ".repeat(indent + 1) + name.as_str()
            }
            LeafData::Inpt(prompt) => {
                " ".repeat(indent)
                    + "Inpt\n"
                    + &" ".repeat(indent + 1)
                    + "\""
                    + prompt.as_str()
                    + "\""
            }
            LeafData::Nmbr(num) => {
                " ".repeat(indent) + "Nmbr\n" + &" ".repeat(indent + 1) + &num.to_string()
            }
        }
    }
}

#[derive(PartialEq, Eq, Debug)]
enum LeafData {
    Name(String),
    Nmbr(u32),
    Inpt(String),
}

pub struct Prgm {
    main: Blck,
}

impl Ast for Prgm {
    type Output = ();

    fn span(&self) -> Span {
        self.main.span()
    }

    fn parse(tokens: &mut TokenStream) -> Result<Self> {
        Ok(Self {
            main: Blck::parse(tokens)?,
        })
    }

    fn eval(self, ctx: &mut Context) -> Result<()> {
        self.main.eval(ctx)
    }

    fn dump(&self, indent: usize) -> String {
        " ".repeat(indent) + "Prgm\n" + &self.main.dump(indent + 1)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    mod leaf {
        use super::*;
        use crate::tokenizer::Tokenizer;

        mod dump {
            use super::*;

            macro_rules! dump_test {
                ($name:ident: $in:expr, $indent:expr => $out:expr) => {
                    #[test]
                    fn $name() {
                        let mut tokens = Tokenizer::lex($in).unwrap();
                        let leaf = Leaf::parse(&mut tokens).unwrap();
                        assert_eq!(leaf.dump($indent).as_str(), $out)
                    }
                };
                ($name:ident: $in:expr => $out:expr) => { dump_test!($name: $in, 0 => $out); }
            }

            dump_test!(num: "3" => "Nmbr\n 3");
            dump_test!(num_indented: "3", 2 => "  Nmbr\n   3");
        }
    }

    mod expn {
        use super::*;
        use crate::tokenizer::Tokenizer;

        // macro_rules! expn_test {
        //     ($name:ident: $in:expn => $out:expn ) => ()
        // }
        mod dump {
            use super::*;

            macro_rules! dump_test {
                ($name:ident: $in:expr, $indent:expr => $out:expr) => {
                    #[test]
                    fn $name() {
                        let mut tokens = Tokenizer::lex($in).unwrap();
                        let leaf = Expn::parse(&mut tokens).unwrap();
                        assert_eq!(leaf.dump($indent).as_str(), $out)
                    }
                };
                ($name:ident: $in:expr => $out:expr) => { dump_test!($name: $in, 0 => $out); }
            }

            dump_test!(plus: "3 + 2" => "Plus\n Nmbr\n  3\n Nmbr\n  2");
        }

        mod eval {
            use super::*;

            macro_rules! eval_test {
                ($name:ident: $in:expr => $out:expr) => {
                    #[test]
                    fn $name() {
                        let mut tokens = Tokenizer::lex($in).unwrap();
                        let expn = Expn::parse(&mut tokens).unwrap();
                        assert_eq!(expn.eval(&mut Context::default()).unwrap(), $out);
                    }
                };
            }

            eval_test!(minus_assoc: "1 - 2 - 3" => -4);
            eval_test!(minus_grouped: "1 - (2 - 3)" => 2);
            eval_test!(two_plus_two: "2+2" => 4);
            eval_test!(mod_assoc: "(2*3) + 14 % 10" => 10);
            eval_test!(expt_assoc: "4 ** 3 ** 2" => 262_144);
            eval_test!(expt_paren: "(4 ** 3) ** 2" => 4096);
            eval_test!(div: "4 // 3" => 1);
            eval_test!(harder_div: "30 // 7" => 4);
            eval_test!(times_precedence: "2 + 5 * 6" => 32);
            eval_test!(expt_and_times_precedence: "1 ** 2 + 5 * 6" => 31);
            eval_test!(double_expt_and_times_precedence: "3 ** 1 ** 2 + 5 * 6" => 33);
            eval_test!(times_and_div_precedence: "5 * 6 // 7" => 4);
            eval_test!(perry_hard: "3 ** 1 ** 2 + 5 * 6 // 7" => 7);
        }

        mod parse {
            use super::*;

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

                assert_eq!(
                    expn,
                    Expn::BinOp {
                        left: num!(1,1 => 2),
                        op: BinOp::Plus,
                        right: num!(1,3 => 2),
                    }
                );
            }

            #[test]
            fn plus_assoc() {
                let mut tokens = Tokenizer::lex("2+3+4").unwrap();
                let expn = Expn::parse(&mut tokens).unwrap();

                assert_eq!(
                    expn,
                    Expn::BinOp {
                        left: Box::new(Expn::BinOp {
                            left: num!(1,1 => 2),
                            op: BinOp::Plus,
                            right: num!(1,3 => 3),
                        }),
                        op: BinOp::Plus,
                        right: num!(1,5 => 4),
                    }
                );
            }

            #[test]
            fn two_plus_two_times_four() {
                let mut tokens = Tokenizer::lex("2+2*4").unwrap();
                let expn = Expn::parse(&mut tokens).unwrap();

                assert_eq!(
                    expn,
                    Expn::BinOp {
                        left: num!(1,1 => 2),
                        op: BinOp::Plus,
                        right: Box::new(Expn::BinOp {
                            left: num!(1,3 => 2),
                            op: BinOp::Times,
                            right: num!(1,5 => 4),
                        })
                    }
                );
            }

            #[test]
            fn paren_two_plus_two_times_four() {
                let mut tokens = Tokenizer::lex("(2+2)*4").unwrap();
                let expn = Expn::parse(&mut tokens).unwrap();

                assert_eq!(
                    expn,
                    Expn::BinOp {
                        left: Box::new(Expn::BinOp {
                            left: num!(1,2 => 2),
                            op: BinOp::Plus,
                            right: num!(1,4 => 2),
                        }),
                        op: BinOp::Times,
                        right: num!(1,7 => 4),
                    }
                );
            }

            #[test]
            fn assgn() {
                let mut tokens = Tokenizer::lex("x=2+2").unwrap();
                let stmt = Stmt::parse(&mut tokens).unwrap();

                assert_eq!(
                    stmt,
                    Stmt {
                        span: Span {
                            start: Loc { row: 1, col: 1 },
                            end: Loc { row: 1, col: 5 },
                        },
                        data: StmtData::Asgn(
                            "x".to_string(),
                            Expn::BinOp {
                                left: num!(1,3 => 2),
                                op: BinOp::Plus,
                                right: num!(1,5 => 2)
                            }
                        ),
                    }
                );
            }
        }
    }
}
