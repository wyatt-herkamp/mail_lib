/*!
 * Parsers for [RFC 2822](https://datatracker.ietf.org/doc/html/rfc2822)
 */
use chumsky::prelude::*;

use super::rfc2234::{self, alpha, crlf, digit, dquote, wsp};
use crate::{mail_box::RawMailBox, parsers::ErrType};

/// [Folding Whitespace Defined in RFC 2822](https://datatracker.ietf.org/doc/html/rfc2822#section-3.2.3)
pub fn fws<'a>() -> impl Parser<'a, &'a str, Vec<char>, ErrType<'a>> {
    let rfc2822_fws = {
        let wsp_then_crlf = wsp().repeated().collect::<Vec<_>>().then_ignore(crlf());
        wsp_then_crlf
            .then(wsp().repeated().at_least(1).collect::<Vec<_>>())
            .map(|(mut wsp_then_crlf, wsp)| {
                wsp_then_crlf.extend(wsp);
                wsp_then_crlf
            })
    };
    // TODO Support obs_fws
    rfc2822_fws
}
/// FWS but it just returns the number of spaces
pub fn fws_counted<'a>() -> impl Parser<'a, &'a str, usize, ErrType<'a>> {
    let rfc2822_fws = {
        let wsp_then_crlf = wsp().repeated().collect::<Vec<_>>().then_ignore(crlf());
        wsp_then_crlf
            .then(wsp().repeated().at_least(1).collect::<Vec<_>>())
            .map(|(wsp_then_crlf, wsp)| wsp_then_crlf.len() + wsp.len())
    };
    // TODO Support obs_fws
    rfc2822_fws
}
pub fn quoted_pair<'a>() -> impl Parser<'a, &'a str, char, ErrType<'a>> {
    just('\\')
        .then(rfc2234::char())
        .map(|v| v.1)
        .or_not()
        .map(|v| v.unwrap_or('\\'))
}
pub fn no_ws_ctl<'a>() -> impl Parser<'a, &'a str, char, ErrType<'a>> {
    choice((
        one_of('\x01'..='\x08'),
        one_of('\x0B'..='\x0C'),
        one_of('\x0E'..='\x1F'),
        just('\x7F'),
    ))
}
pub fn obs_qp<'a>() -> impl Parser<'a, &'a str, char, ErrType<'a>> {
    just('\\')
        .then(rfc2234::char())
        .map(|v| v.1)
        .or_not()
        .map(|v| v.unwrap_or('\\'))
}
pub fn obs_char<'a>() -> impl Parser<'a, &'a str, char, ErrType<'a>> {
    choice((
        one_of('\x00'..='\x09'),
        just('\x0B'),
        just('\x0C'),
        one_of('\x0E'..='\x7F'),
    ))
}
pub fn obs_text<'a>() -> impl Parser<'a, &'a str, Vec<char>, ErrType<'a>> {
    let lf_cr = rfc2234::lf()
        .repeated()
        .ignored()
        .then(rfc2234::cr().repeated().ignored());

    let obs_chars = obs_char()
        .then_ignore(
            rfc2234::lf()
                .repeated()
                .ignored()
                .then(rfc2234::cr().repeated().ignored()),
        )
        .repeated()
        .collect::<Vec<_>>();

    lf_cr.then(obs_chars).map(|(_, s)| s)
}
pub fn text<'a>() -> impl Parser<'a, &'a str, char, ErrType<'a>> {
    choice((
        crlf().map(|_| '\n'),
        one_of('\x01'..='\x09'),
        one_of('\x0B'..='\x0C'),
        one_of('\x0E'..='\x7F'),
    ))
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
pub fn cfws<'a>() -> impl Parser<'a, &'a str, String, ErrType<'a>> {
    // TODO Support Comments
    fws().map(|v| v.into_iter().collect::<String>())
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
/// An Atom
pub fn atom<'a>() -> impl Parser<'a, &'a str, String, ErrType<'a>> {
    cfws()
        .ignored()
        .then(atext().repeated().at_least(1).collect::<String>())
        .then_ignore(cfws())
        .map(|(_, atext)| atext)
}

pub fn dot_atom_text<'a>() -> impl Parser<'a, &'a str, String, ErrType<'a>> {
    atext()
        .repeated()
        .at_least(1)
        .collect::<String>()
        .separated_by(just('.'))
        .collect::<Vec<_>>()
        .map(|v| v.join("."))
}
///
/// ```ebnf
/// qtext           =       NO-WS-CTL /     ; Non white space controls
///
///                        %d33 /          ; The rest of the US-ASCII
///                        %d35-91 /       ;  characters not including "\"
///                        %d93-126        ;  or the quote character
/// ```
pub fn qtext<'a>() -> impl Parser<'a, &'a str, char, ErrType<'a>> {
    choice((
        no_ws_ctl(),
        just('\x21'),
        one_of('\x23'..='\x5B'),
        one_of('\x5D'..='\x7E'),
    ))
}
/// ```ebnf
/// qcontent        =       qtext / quoted-pair
/// ```
pub fn qcontent<'a>() -> impl Parser<'a, &'a str, char, ErrType<'a>> {
    qtext().or(quoted_pair()).or_not().map(|v| v.unwrap_or(' '))
}
/// ```ebnf
/// quoted-string   =       [CFWS]
///                        DQUOTE *([FWS] qcontent) [FWS] DQUOTE
///                       [CFWS]
/// ```
pub fn quoted_string<'a>() -> impl Parser<'a, &'a str, String, ErrType<'a>> {
    let inner = fws_counted()
        .or_not()
        .then(qcontent())
        .repeated()
        .collect::<Vec<_>>()
        .map(|v| {
            let mut s = String::with_capacity(v.len());
            for (spaces, c) in v {
                if let Some(spaces) = spaces {
                    s.push_str(&" ".repeat(spaces));
                }
                s.push(c);
            }
            s
        });

    cfws()
        .or_not()
        .ignored()
        .then_ignore(dquote())
        .then(inner)
        .then_ignore(fws().or_not())
        .then_ignore(dquote())
        .then_ignore(cfws().or_not())
        .map(|(_, content)| content)
}
pub fn word<'a>() -> impl Parser<'a, &'a str, String, ErrType<'a>> {
    choice((quoted_string(), atom()))
}

pub fn obs_phrase<'a>() -> impl Parser<'a, &'a str, String, ErrType<'a>> {
    let obs_phrase_part_two = choice((word(), just('.').map(|v| v.to_string()), cfws()))
        .repeated()
        .collect::<Vec<_>>();
    word().then(obs_phrase_part_two).map(|(v, two)| {
        let mut s = v;
        for v in two {
            s.push_str(&v);
        }
        s
    })
}
pub fn pharse<'a>() -> impl Parser<'a, &'a str, String, ErrType<'a>> {
    choice((
        word()
            .repeated()
            .at_least(1)
            .collect::<Vec<_>>()
            .map(|v| v.into_iter().collect::<Vec<_>>().join("")),
        obs_phrase(),
    ))
}

pub fn dot_atom<'a>() -> impl Parser<'a, &'a str, String, ErrType<'a>> {
    cfws()
        .or_not()
        .ignored()
        .then(dot_atom_text())
        .then_ignore(cfws().or_not())
        .map(|(_, atext)| atext)
}
pub fn local_part<'a>() -> impl Parser<'a, &'a str, String, ErrType<'a>> {
    choice((dot_atom(), quoted_string()))
}
pub fn domain<'a>() -> impl Parser<'a, &'a str, String, ErrType<'a>> {
    choice((dot_atom(), domain_literal()))
}
pub fn domain_literal<'a>() -> impl Parser<'a, &'a str, String, ErrType<'a>> {
    todo()
}
pub fn addr_spec<'a>() -> impl Parser<'a, &'a str, (String, String), ErrType<'a>> {
    local_part().then_ignore(just('@')).then(domain())
}
pub fn angle_addr<'a>() -> impl Parser<'a, &'a str, (String, String), ErrType<'a>> {
    just('<').ignore_then(addr_spec()).then_ignore(just('>'))
}
pub fn display_name<'a>() -> impl Parser<'a, &'a str, String, ErrType<'a>> {
    pharse()
}
pub fn name_addr<'a>() -> impl Parser<'a, &'a str, RawMailBox, ErrType<'a>> {
    display_name()
        .or_not()
        .padded()
        .then(angle_addr())
        .map(|(display_name, addr)| RawMailBox {
            display_name,
            local: addr.0,
            domain: addr.1,
        })
}

pub fn mailbox<'a>() -> impl Parser<'a, &'a str, RawMailBox, ErrType<'a>> {
    choice((
        name_addr(),
        addr_spec().map(|(local, domain)| RawMailBox {
            display_name: None,
            local,
            domain,
        }),
    ))
}
