use chumsky::prelude::*;

use crate::parsers::ErrType;

/// ```ebnf
/// vchar           =  %x21-7E ; visible (printing) characters
/// ```
pub fn vchar<'a>() -> impl Parser<'a, &'a str, char, ErrType<'a>> {
    one_of('\x21'..='\x7E')
}

pub use super::rfc2234::{cr, crlf, dquote, lf, wsp};
