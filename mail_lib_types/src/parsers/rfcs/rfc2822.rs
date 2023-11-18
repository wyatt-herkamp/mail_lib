/*!
 * Parsers for [RFC 2822](https://datatracker.ietf.org/doc/html/rfc2234)
 */
use chumsky::prelude::*;

use super::rfc2234::{self, alpha, crlf, digit, wsp};
use crate::parsers::ErrType;

/// [Folding Whitespace Defined in RFC 2822](https://datatracker.ietf.org/doc/html/rfc2822#section-3.2.3)
pub fn fws<'a>() -> impl Parser<'a, &'a str, Option<char>, ErrType<'a>> {
    wsp().or_not().then(wsp().ignored().repeated()).map(|v| v.0)
}
pub fn quoted_pair<'a>() -> impl Parser<'a, &'a str, char, ErrType<'a>> {
    just('\\')
        .then(rfc2234::char())
        .map(|v| v.1)
        .or_not()
        .map(|v| v.unwrap_or('\\'))
}
pub fn ctext<'a>() -> impl Parser<'a, &'a str, char, ErrType<'a>> {
    choice((
        // TODO: fix this
        alpha(),
        digit(),
    ))
}

pub fn ccontent<'a>() -> impl Parser<'a, &'a str, Option<String>, ErrType<'a>> {
    todo()
}
// TODO: fix this
pub fn comment<'a>() -> impl Parser<'a, &'a str, String, ErrType<'a>> {
    todo()
}
pub fn cfws<'a>() -> impl Parser<'a, &'a str, Option<char>, ErrType<'a>> {
    fws()
}
/// [atext Defined in RFC 2822](https://datatracker.ietf.org/doc/html/rfc2822#section-3.2.4)
pub fn atext<'a>() -> impl Parser<'a, &'a str, char, ErrType<'a>> {
    choice((
        alpha(),
        digit(),
        just('!'),
        just('#'),
        just('$'),
        just('%'),
        just('&'),
        just('\''),
        just('*'),
        just('+'),
        just('-'),
        just('/'),
        just('='),
        just('?'),
        just('^'),
        just('_'),
        just('`'),
        just('{'),
        just('|'),
        just('}'),
        just('~'),
    ))
}

pub fn atom<'a>() -> impl Parser<'a, &'a str, String, ErrType<'a>> {
    cfws()
        .ignored()
        .then(atext().repeated().at_least(1).collect::<String>())
        .then_ignore(cfws())
        .map(|(_, atext)| atext)
}

pub fn dot_atom_text<'a>() -> impl Parser<'a, &'a str, String, ErrType<'a>> {
    todo()
}
