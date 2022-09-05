//! The lexer.

pub struct Loc {
    row: usize,
    col: usize,
}

pub struct Span {
    start: Loc,
    end: Loc,
}

pub(crate) struct Token<'a> {
    kind: TokenKind<'a>,
    span: Span,
}

#[derive(PartialEq, Eq)]
pub(crate) enum TokenKind<'a> {
    NewLine,
    Name(&'a str),
    Number(i32),
}

pub(crate) struct TokenStream<'a> {
    tokens: Vec<Token<'a>>,
    index: usize,
}

impl<'a> TokenStream<'a> {
    /// Append to the token.
    pub fn append(&mut self, tkn: Token<'a>) {
        self.tokens.push(tkn);
    }

    /// Reset to the beginning of the tokens.
    pub fn reset(&mut self) {
        self.index = 0;
    }

    /// Return the current token.
    pub fn current(&self) -> &'a Token {
        &self.tokens[self.index]
    }

    /// Advance the token stream.
    pub fn advance(&mut self) {
        assert!(self.index < self.tokens.len());
        self.index += 1;
    }

    /// Eat a token of `TokenKind`.
    pub fn eat(&mut self, target: &TokenKind) {
        assert!(target == &self.current().kind);
        self.advance();
    }
}

pub(crate) struct Tokenizer<'a> {
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
    fn curr_char(&self) -> char {
        self.source
            .lines()
            .nth(self.loc.row - 1)
            .expect("We are at a valid row.")
            .chars()
            .nth(self.loc.col - 1)
            .expect("We are at a valid column.")
    }

    /// Advance the pointer one character.
    fn advance(&mut self) {
        if self.curr_char() == '\n' {
            self.row += 1;
            self.col = 1;
        } else {
            self.col += 1;
        }
    }
}

/// Possible states of the state machine.
enum TokenizerState {
    INIT,
    WTHN,
    TABS,
    CMMT_INIT,
    CMMT_WTHN,
    NMBR,
    STRG,
    ESCP,
    SLSH,
    IDEN_RSRV,
    HALT,
}
