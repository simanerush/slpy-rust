//! The lexer.

use crate::error::{Error, Kind, Result};
use crate::{Loc, Span};

#[derive(PartialEq, Eq, Debug)]
pub struct Token {
    pub kind: TokenKind,
    pub span: Span,
}

#[derive(PartialEq, Eq, Debug)]
pub enum TokenKind {
    NewLine,
    Ident(String),
    Number(u32),
    Str(String),
    LParen,
    RParen,
    Op(Op),
}

#[derive(PartialEq, Eq, Debug, Copy, Clone)]
pub enum Op {
    Plus,
    Minus,
    Times,
    Div,
    Mod,
    Eq,
}

#[derive(Default, PartialEq, Eq, Debug)]
pub struct TokenStream {
    tokens: Vec<Token>,
    index: usize,
}

impl TokenStream {
    /// Append to the token.
    pub fn append(&mut self, tkn: Token) {
        self.tokens.push(tkn);
    }

    /// Reset to the beginning of the tokens.
    pub fn reset(&mut self) {
        self.index = 0;
    }

    /// Return the current token.
    #[must_use]
    pub fn current(&self) -> Option<&Token> {
        self.tokens.get(self.index)
    }

    pub fn current_or(&self) -> Result<&Token> {
        self.current().ok_or(Error {
            span: Span {
                start: self.eof(),
                end: self.eof(),
            },
            kind: Kind::UnexpectedEof,
        })
    }

    // Get the location of the end of file.
    fn eof(&self) -> Loc {
        self.tokens
            .last()
            .map_or(Loc { row: 1, col: 1 }, |t| t.span.end)
    }

    /// Advance the token stream.
    pub fn advance(&mut self) {
        assert!(self.index < self.tokens.len());
        self.index += 1;
    }

    /// Eat a token of `TokenKind`.
    pub fn eat(&mut self, target: &TokenKind) {
        assert!(&self.current().map_or(false, |i| &i.kind == target));
        self.advance();
    }

    /// Take the current token from the tokenizer.
    pub fn take(&mut self) -> Token {
        self.tokens.remove(self.index)
    }
}

pub struct Tokenizer<'a> {
    loc: Loc,
    source: &'a str,
}

impl<'a> Tokenizer<'a> {
    /// Create a new tokenizer from the source string.
    pub const fn new(source: &'a str) -> Self {
        Self {
            loc: Loc { row: 1, col: 1 },
            source,
        }
    }

    /// Get the current character.
    ///
    /// Returns `None` if we're done with the string.
    fn curr_char(&self) -> Option<char> {
        self.source
            .lines()
            .nth(self.loc.row - 1)?
            .chars()
            .chain(std::iter::once('\n')) // force every line to end with a newline
            .nth(self.loc.col - 1)
    }

    /// Advance the pointer one character.
    fn advance(&mut self) {
        if self.curr_char() == Some('\n') {
            self.loc.row += 1;
            self.loc.col = 1;
        } else {
            self.loc.col += 1;
        }
    }

    /// Parse a token consisting of a single character.
    fn single_char(&mut self, kind: TokenKind) -> Token {
        let token = Token {
            kind,
            span: Span {
                start: self.loc,
                end: self.loc,
            },
        };
        self.advance();
        token
    }

    /// Parse a token consisting of two characters.
    fn expect_next(&mut self, kind: TokenKind, next: char) -> Result<Token> {
        let start = self.loc;
        self.advance();

        let span = Span {
            start,
            end: self.loc,
        };

        if self.curr_char() == Some(next) {
            Ok(Token { kind, span })
        } else {
            Err(Error {
                kind: Kind::Tokenization,
                span,
            })
        }
    }

    /// Parse a token while a condition holds true.
    ///
    /// Accum should update some internal state for building the token and return whether to
    /// continue parsing. Finally should turn the state into a `TokenKind`.
    fn parse_while<T>(
        &mut self,
        mut init: T,
        mut accum: impl FnMut(&mut T, char) -> bool,
        finally: impl FnOnce(T) -> TokenKind,
    ) -> Token {
        let start = self.loc;
        let mut end = self.loc;

        while let Some(c) = self.curr_char() {
            if !accum(&mut init, c) {
                break;
            }
            end = self.loc;
            self.advance();
        }

        Token {
            kind: finally(init),
            span: Span { start, end },
        }
    }

    /// Parse the next token from the string.
    fn next_token(&mut self) -> Result<Option<Token>> {
        #[allow(clippy::enum_glob_use)]
        use self::Op::*;
        #[allow(clippy::enum_glob_use)]
        use TokenKind::*;

        // skip whitespace that we don't care about
        while self.curr_char() == Some(' ') || self.curr_char() == Some('\t') {
            self.advance();
        }

        Ok(if let Some(c) = self.curr_char() {
            Some(match c {
                '\n' => self.single_char(NewLine),
                '(' => self.single_char(LParen),
                ')' => self.single_char(RParen),
                '+' => self.single_char(Op(Plus)),
                '-' => self.single_char(Op(Minus)),
                '*' => self.single_char(Op(Times)),
                '/' => self.expect_next(Op(Div), '/')?,
                '%' => self.single_char(Op(Mod)),
                '=' => self.single_char(Op(Eq)),
                '#' => {
                    self.loc.row += 1;
                    self.loc.col = 1;
                    return self.next_token();
                }
                '0'..='9' => self.parse_while(
                    0,
                    |n, c| {
                        c.to_digit(10).map_or(false, |d| {
                            *n *= 10;
                            *n += d;
                            true
                        })
                    },
                    Number,
                ),
                'a'..='z' | 'A'..='Z' | '_' => self.parse_while(
                    String::new(),
                    |s, c| {
                        if c.is_alphanumeric() || c == '_' {
                            s.push(c);
                            true
                        } else {
                            false
                        }
                    },
                    Ident,
                ),
                '"' => {
                    let mut seen_even_quotes = true;
                    self.parse_while(
                        String::new(),
                        |s, c| {
                            if c == '"' {
                                // this will always be set to false immediately
                                seen_even_quotes = !seen_even_quotes;

                                // always return true here - we need to advance past the second "
                                true
                            } else if seen_even_quotes {
                                // here we've seen two quotes, so we're done
                                false
                            } else {
                                s.push(c);
                                true
                            }
                        },
                        Str,
                    )
                }
                _ => todo!(),
            })
        } else {
            None
        })
    }

    /// Lex source into a `TokenStream`.
    pub fn lex(source: &'a str) -> Result<TokenStream> {
        let mut tokenizer = Self::new(source);
        let mut tokens = TokenStream::default();

        while let Some(tkn) = tokenizer.next_token()? {
            tokens.append(tkn);
        }

        Ok(tokens)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    mod next_token {
        use super::*;

        #[allow(clippy::enum_glob_use)]
        use super::Op::*;
        #[allow(clippy::enum_glob_use)]
        use TokenKind::*;

        /// A `next_token` test case.
        macro_rules! ntt {
            ($name:ident: $in:expr => $out:expr) => {
                #[test]
                fn $name() -> Result<()> {
                    assert_eq!(Tokenizer::new($in).next_token()?.unwrap().kind, $out);
                    Ok(())
                }
            };
        }

        ntt!(l_paren: "(" => LParen);
        ntt!(r_paren: ")" => RParen);
        ntt!(newline: "\n" => NewLine);
        ntt!(plus: "+" => Op(Plus));
        ntt!(minus: "-" => Op(Minus));
        ntt!(times: "*" => Op(Times));
        ntt!(div: "//" => Op(Div));
        ntt!(modulus: "%" => Op(Mod));
        ntt!(eq: "=" => Op(Eq));
        ntt!(comment: "#\nx" => Ident("x".to_string()));
        ntt!(num: "1234" => Number(1234));
        ntt!(ident: "abcd" => Ident("abcd".to_string()));
        ntt!(ident_underscore: "_abcd" => Ident("_abcd".to_string()));
        ntt!(ident_numbers: "a_124_Bb41" => Ident("a_124_Bb41".to_string()));
        ntt!(str1: "\"a b c\"" => Str("a b c".to_string()));
    }

    mod lex {
        use super::*;

        #[allow(clippy::enum_glob_use)]
        use super::Op::*;
        #[allow(clippy::enum_glob_use)]
        use TokenKind::*;

        macro_rules! tok {
            ($srow:expr,$scol:expr; $erow:expr,$ecol:expr => $kind:expr) => {
                Token {
                    kind: $kind,
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
            };
            ($srow:expr,$scol:expr => $kind:expr) => {tok!($srow,$scol;$srow,$scol => $kind)}
        }

        /// A `lex` test case.
        macro_rules! lt {
            ($name:ident: $in:expr => $($out:expr),*) => {
                #[test]
                fn $name() -> Result<()> {
                    assert_eq!(Tokenizer::lex($in)?.tokens, vec![$($out),*]);
                    Ok(())
                }
            };
        }

        lt! {call: "print(x)" =>
            tok!(1,1;1,5 => Ident("print".to_string())),
            tok!(1,6 => LParen),
            tok!(1,7 => Ident("x".to_string())),
            tok!(1,8 => RParen),
            tok!(1,9 => NewLine)
        }

        lt! {mult: "a_b*y" =>
            tok!(1,1;1,3 => Ident("a_b".to_string())),
            tok!(1,4 => Op(Times)),
            tok!(1,5 => Ident("y".to_string())),
            tok!(1,6 => NewLine)
        }

        lt! {mult_whitespace: "a_b * y" =>
            tok!(1,1;1,3 => Ident("a_b".to_string())),
            tok!(1,5 => Op(Times)),
            tok!(1,7 => Ident("y".to_string())),
            tok!(1,8 => NewLine)
        }

        lt! {two_lines: "x = input(\"Enter a number.\")\nprint(x)" =>
            tok!(1,1 => Ident("x".to_string())),
            tok!(1,3 => Op(Eq)),
            tok!(1,5;1,9 => Ident("input".to_string())),
            tok!(1,10 => LParen),
            tok!(1,11;1,27 => Str("Enter a number.".to_string())),
            tok!(1,28 => RParen),
            tok!(1,29 => NewLine),
            tok!(2,1;2,5 => Ident("print".to_string())),
            tok!(2,6 => LParen),
            tok!(2,7 => Ident("x".to_string())),
            tok!(2,8 => RParen),
            tok!(2,9 => NewLine)
        }

        // TODO: whitespace
    }
}
