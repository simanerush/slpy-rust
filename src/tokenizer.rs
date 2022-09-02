//! The lexer.

struct Token<'a> {
    kind: TokenKind<'a>,
    row: u32,
    col: u32,
}

#[derive(PartialEq, Eq)]
enum TokenKind<'a> {
    NewLine,
    Name(&'a str),
    Number(i32),
    String(&'a str),
}

struct TokenStream<'a> {
    tokens: Vec<Token<'a>>,
    index: usize,
}

impl<'a> TokenStream<'a> {
    /// Append to the token.
    pub(crate) fn append(&mut self, tkn: Token<'a>) {
        self.tokens.push(tkn);
    }

    /// Reset to the beginning of the tokens.
    pub(crate) fn reset(&mut self) {
        self.index = 0;
    }

    /// Return the current token.
    pub(crate) fn current(&self) -> &'a Token {
        &self.tokens[self.index]
    }

    /// Advance the token stream.
    pub(crate) fn advance(&mut self) {
        assert!(self.index < self.tokens.len());
        self.index += 1;
    }

    /// Eat a token of `TokenKind`.
    pub(crate) fn eat(&mut self, target: &TokenKind) {
        assert!(target == &self.current().kind);
        self.advance();
    }
}
