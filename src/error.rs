// TODO: error
use crate::Span;

use thiserror::Error;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, PartialEq, Eq, Error)]
#[error("{kind} from {} to {}", .span.start, .span.end)]
pub struct Error {
    pub kind: Kind,
    pub span: Span,
}

#[derive(Debug, PartialEq, Eq, thiserror::Error)]
pub enum Kind {
    #[error("tokenization failed")]
    Tokenization,

    #[error("parsing failed")]
    Parser,

    #[error("unexpected end of file")]
    UnexpectedEof,
}
