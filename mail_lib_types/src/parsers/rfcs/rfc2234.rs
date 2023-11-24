/*!
 * Parsers for [RFC 2234](https://datatracker.ietf.org/doc/html/rfc2234)
 */
use chumsky::prelude::*;

use crate::parsers::ErrType;

/// [Alpha Defined in RFC 2234](https://datatracker.ietf.org/doc/html/rfc2234#section-6.1)
/// ```ebnf
/// ALPHA          =  %x41-5A / %x61-7A   ; A-Z / a-z
/// ```
pub fn alpha<'a>() -> impl Parser<'a, &'a str, char, ErrType<'a>> {
    choice((one_of('a'..='z'), one_of('A'..='Z')))
}
/// [Char Defined in RFC 2234](https://datatracker.ietf.org/doc/html/rfc2234#section-6.1)
/// ```ebnf
/// CHAR =  %x01-7F ; any 7-bit US-ASCII character, excluding NUL
/// ```
pub fn char<'a>() -> impl Parser<'a, &'a str, char, ErrType<'a>> {
    one_of('\x01'..='\x7F')
}

/// [CR Defined in RFC 2234](https://datatracker.ietf.org/doc/html/rfc2234#section-6.1)
/// ```ebnf
/// CR =  %x0D ; carriage return
/// ```
pub fn cr<'a>() -> impl Parser<'a, &'a str, char, ErrType<'a>> {
    just('\r')
}
/// [LF Defined in RFC 2234](https://datatracker.ietf.org/doc/html/rfc2234#section-6.1)
/// ```ebnf
/// LF =  %x0A ; linefeed
/// ```
pub fn lf<'a>() -> impl Parser<'a, &'a str, char, ErrType<'a>> {
    just('\n')
}

/// [CRLF Defined in RFC 2234](https://datatracker.ietf.org/doc/html/rfc2234#section-6.1)
/// ```ebnf
/// CRLF =  CR LF ; Internet standard newline
/// ```
pub fn crlf<'a>() -> impl Parser<'a, &'a str, String, ErrType<'a>> {
    just("\r\n").map(|v| v.to_string())
}

/// [CTL Defined in RFC 2234](https://datatracker.ietf.org/doc/html/rfc2234#section-6.1)
/// ```ebnf
/// CTL =  %x00-1F / %x7F ; controls
/// ```
pub fn ctl<'a>() -> impl Parser<'a, &'a str, char, ErrType<'a>> {
    choice((one_of('\x00'..='\x1F'), just('\x7F')))
}

/// [DIGIT Defined in RFC 2234](https://datatracker.ietf.org/doc/html/rfc2234#section-6.1)
/// ```ebnf
/// DIGIT =  %x30-39 ; 0-9
/// ```
pub fn digit<'a>() -> impl Parser<'a, &'a str, char, ErrType<'a>> {
    one_of('0'..='9')
}

/// [DQUOTE Defined in RFC 2234](https://datatracker.ietf.org/doc/html/rfc2234#section-6.1)
/// ```ebnf
/// DQUOTE =  %x22 ; " (Double Quote)
/// ```
pub fn dquote<'a>() -> impl Parser<'a, &'a str, char, ErrType<'a>> {
    just('"')
}

/// [HexDigit Defined in RFC 2234](https://datatracker.ietf.org/doc/html/rfc2234#section-6.1)
/// ```ebnf
/// HEXDIG =  DIGIT / "A" / "B" / "C" / "D" / "E" / "F"
/// ```
pub fn hexdig<'a>() -> impl Parser<'a, &'a str, char, ErrType<'a>> {
    choice((digit(), one_of('A'..='F')))
}

/// [HTAB Defined in RFC 2234](https://datatracker.ietf.org/doc/html/rfc2234#section-6.1)
/// ```ebnf
/// HTAB =  %x09 ; horizontal tab
/// ```
pub fn htab<'a>() -> impl Parser<'a, &'a str, char, ErrType<'a>> {
    just('\x09')
}
/// [Whitespace Defined in RFC 2234](https://datatracker.ietf.org/doc/html/rfc2234#section-6.1)
/// ```ebnf
/// WSP =  SP / HTAB ; white space
/// ```
pub fn wsp<'a>() -> impl Parser<'a, &'a str, char, ErrType<'a>> {
    just(' ').or(htab())
}
/// [LWSP Defined in RFC 2234](https://datatracker.ietf.org/doc/html/rfc2234#section-6.1)
/// ```ebnf
/// LWSP =  *(WSP / CRLF WSP) ; Use of this linear-white-space rule
/// ```
pub fn lwsp<'a>() -> impl Parser<'a, &'a str, String, ErrType<'a>> {
    wsp().then(crlf()).repeated().collect::<Vec<_>>().map(|v| {
        let mut s = String::new();
        for (wsp, crlf) in v {
            s.push(wsp);
            s.push_str(crlf.as_str())
        }
        s
    })
}
#[cfg(test)]
mod tests {
    use chumsky::Parser;

    use pretty_assertions::assert_eq;
    #[test]
    pub fn wsp() {
        let parser = super::wsp();
        let result = parser.parse(" ").into_result();
        assert_eq!(result, Ok(' '));
        let result = parser.parse("\t").into_result();
        assert_eq!(result, Ok('\t'));
    }
}
