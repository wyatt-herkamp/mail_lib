/*!
 * Parsers for [RFC 2822](https://datatracker.ietf.org/doc/html/rfc2822)
 */

use chumsky::prelude::*;

use super::rfc2234::{self, alpha, crlf, digit, dquote, wsp};
use crate::{mail_box::RawMailBox, parsers::ErrType};

/// [Folding Whitespace Defined in RFC 2822](https://datatracker.ietf.org/doc/html/rfc2822#section-3.2.3)
/// ```ebnf
///    FWS             =   ([*WSP CRLF] 1*WSP) /  obs-FWS
/// ```
pub fn fws<'a>() -> impl Parser<'a, &'a str, &'a str, ErrType<'a>> {
    let rfc2822_fws = {
        let wsp_then_crlf = wsp().repeated().then(crlf()).to_slice().or_not();
        wsp_then_crlf
            .then(wsp().repeated().at_least(1).to_slice())
            .to_slice()
    };
    // TODO Support obs_fws
    rfc2822_fws
}
/// ```ebnf
/// quoted-pair     =       ("\" (TEXT / WSP))
/// ```
pub fn quoted_pair<'a>() -> impl Parser<'a, &'a str, &'a str, ErrType<'a>> {
    just('\\').then(text()).to_slice()
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
pub fn cfws<'a>() -> impl Parser<'a, &'a str, &'a str, ErrType<'a>> {
    // TODO Support Comments
    fws()
}
/// [atext Defined in RFC 2822](https://datatracker.ietf.org/doc/html/rfc2822#section-3.2.4)
pub fn atext<'a>() -> impl Parser<'a, &'a str, char, ErrType<'a>> {
    choice((
        // Instead of having a choice inside of a choice call the parser directly
        one_of('a'..='z'),
        one_of('A'..='Z'),
        one_of('0'..='9'),
        one_of([
            '!', '#', '$', '%', '&', '\'', '*', '+', '-', '/', '=', '?', '^', '_', '`', '{', '|',
            '}', '~',
        ]),
    ))
}
pub fn atext_seg<'a>() -> impl Parser<'a, &'a str, &'a str, ErrType<'a>> {
    atext().repeated().at_least(1).to_slice()
}
/// An Atom
pub fn atom<'a>() -> impl Parser<'a, &'a str, &'a str, ErrType<'a>> {
    cfws()
        .or_not()
        .ignore_then(atext_seg())
        .then_ignore(cfws().or_not())
}

/// ```ebnf
/// dot-atom-text = 1*atext *("." 1*atext)
/// ```
pub fn dot_atom_text<'a>() -> impl Parser<'a, &'a str, &'a str, ErrType<'a>> {
    atext_seg().separated_by(just('.')).to_slice()
}
///
/// ```ebnf
/// qtext           =       NO-WS-CTL /     ; Non white space controls
///
///                        %d33 /          ; The rest of the US-ASCII
///                        %d35-91 /       ;  characters not including "\"
///                        %d93-126        ;  or the quote character
/// ```
pub fn qtext<'a>() -> impl Parser<'a, &'a str, &'a str, ErrType<'a>> {
    choice((
        just('\x21'),
        one_of('\x23'..='\x5B'),
        one_of('\x5D'..='\x7E'),
        no_ws_ctl(),
    ))
    .to_slice()
}
/// ```ebnf
/// qcontent        =       qtext / quoted-pair
/// ```
pub fn qcontent<'a>() -> impl Parser<'a, &'a str, &'a str, ErrType<'a>> {
    qtext().or(quoted_pair())
}
/// ```ebnf
/// quoted-string   =       [CFWS]
///                        DQUOTE *([FWS] qcontent) [FWS] DQUOTE
///                       [CFWS]
/// ```
pub fn quoted_string<'a>() -> impl Parser<'a, &'a str, &'a str, ErrType<'a>> {
    let inner = fws()
        .or_not()
        .then(qcontent())
        .repeated()
        .to_slice()
        .delimited_by(dquote(), fws().or_not().then(dquote()))
        .to_slice();

    cfws()
        .or_not()
        .ignore_then(inner)
        .then_ignore(cfws().or_not())
}
pub fn quoted_string_strip_quotes<'a>() -> impl Parser<'a, &'a str, &'a str, ErrType<'a>> {
    let inner = fws()
        .or_not()
        .then(qcontent())
        .repeated()
        .to_slice()
        .delimited_by(dquote(), fws().or_not().then(dquote()));

    cfws()
        .or_not()
        .ignore_then(inner)
        .then_ignore(cfws().or_not())
}
pub fn word<'a>() -> impl Parser<'a, &'a str, &'a str, ErrType<'a>> {
    choice((quoted_string(), atom()))
}
pub fn word_strip_quotes<'a>() -> impl Parser<'a, &'a str, &'a str, ErrType<'a>> {
    choice((quoted_string_strip_quotes(), atom()))
}
/// ```ebnf
/// obs-phrase      =       word *(word / "." / CFWS)
/// ```
pub fn obs_phrase<'a>() -> impl Parser<'a, &'a str, &'a str, ErrType<'a>> {
    let inner = choice((word(), just('.').to_slice(), cfws()))
        .repeated()
        .to_slice();

    word().then(inner).to_slice()
}
pub fn pharse<'a>() -> impl Parser<'a, &'a str, &'a str, ErrType<'a>> {
    // choice((word().map(|v| v.to_owned())))
    word()
}
/// ```ebnf
/// display-name    =       phrase
/// ```
///
/// This does not call `phrase` because I want the `"` to be stripped
pub fn display_name<'a>() -> impl Parser<'a, &'a str, &'a str, ErrType<'a>> {
    word_strip_quotes()
}
pub fn dot_atom<'a>() -> impl Parser<'a, &'a str, &'a str, ErrType<'a>> {
    cfws()
        .or_not()
        .ignore_then(dot_atom_text())
        .then_ignore(cfws().or_not())
}
pub fn local_part<'a>() -> impl Parser<'a, &'a str, &'a str, ErrType<'a>> {
    choice((quoted_string(), dot_atom()))
}
pub fn domain<'a>() -> impl Parser<'a, &'a str, &'a str, ErrType<'a>> {
    choice((domain_literal(), dot_atom()))
}
pub fn dtext<'a>() -> impl Parser<'a, &'a str, &'a str, ErrType<'a>> {
    choice((
        one_of('\x21'..='\x5A'),
        one_of('\x5E'..='\x7E'),
        no_ws_ctl(),
    ))
    .to_slice()
}
pub fn dcontent<'a>() -> impl Parser<'a, &'a str, &'a str, ErrType<'a>> {
    choice((dtext(), quoted_pair()))
}
/// ```ebnf
/// domain-literal  =       [CFWS] "[" *([FWS] dcontent) [FWS] "]" [CFWS]
/// ```
/// Use [rfc5322::domain_literal](super::rfc5322::domain_literal) instead
pub fn domain_literal<'a>() -> impl Parser<'a, &'a str, &'a str, ErrType<'a>> {
    let domain_inner = fws().or_not().ignore_then(dcontent()).repeated().to_slice();
    crlf()
        .or_not()
        .ignore_then(just('['))
        .then(domain_inner)
        .then_ignore(fws().or_not())
        .then(just(']'))
        .then_ignore(crlf().or_not())
        .to_slice()
}
pub fn addr_spec<'a>() -> impl Parser<'a, &'a str, (&'a str, &'a str), ErrType<'a>> {
    local_part().then_ignore(just('@')).then(domain())
}
/// ```ebnf
/// angle-addr      =       [CFWS] "<" addr-spec ">" [CFWS] / obs-angle-addr
/// ```
/// Use [rfc5322::angle_addr](super::rfc5322::angle_addr) instead
pub fn angle_addr<'a>() -> impl Parser<'a, &'a str, (&'a str, &'a str), ErrType<'a>> {
    just('<').ignore_then(addr_spec()).then_ignore(just('>'))
}
/// ```ebnf
/// name-addr       =       [display-name] angle-addr
/// ```
/// Use [rfc5322::name_addr](super::rfc5322::name_addr) instead
pub fn name_addr<'a>() -> impl Parser<'a, &'a str, RawMailBox<'a>, ErrType<'a>> {
    display_name()
        .or_not()
        .padded()
        .then(angle_addr())
        .map(|(display_name, (local, domain))| RawMailBox::new(display_name, local, domain))
}
/// ```ebnf
/// mailbox         =       name-addr / addr-spec
/// ```
/// Use [rfc5322::mailbox](super::rfc5322::mailbox) instead
pub fn mailbox<'a>() -> impl Parser<'a, &'a str, RawMailBox<'a>, ErrType<'a>> {
    choice((
        name_addr(),
        addr_spec().map(|(local, domain)| RawMailBox::new_no_name(local, domain)),
    ))
}
