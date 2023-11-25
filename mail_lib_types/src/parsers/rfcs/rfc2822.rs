/*!
 * Parsers for [RFC 2822](https://datatracker.ietf.org/doc/html/rfc2822)
 */
use chumsky::{container::Container, prelude::*};
use either::Either;

use super::rfc2234::{self, alpha, crlf, digit, dquote, wsp};
use crate::{mail_box::RawMailBox, parsers::ErrType};

/// [Folding Whitespace Defined in RFC 2822](https://datatracker.ietf.org/doc/html/rfc2822#section-3.2.3)
pub fn fws<'a>() -> impl Parser<'a, &'a str, Vec<char>, ErrType<'a>> {
    let rfc2822_fws = {
        let wsp_then_crlf = wsp()
            .repeated()
            .collect::<Vec<_>>()
            .then_ignore(crlf())
            .or_not();
        wsp_then_crlf
            .then(wsp().repeated().at_least(1).collect::<Vec<_>>())
            .map(|(wsp_then_crlf, wsp)| {
                if let Some(mut wsp_then_crlf) = wsp_then_crlf {
                    wsp_then_crlf.extend(wsp);
                    wsp_then_crlf
                } else {
                    wsp
                }
            })
    };
    // TODO Support obs_fws
    rfc2822_fws
}
/// FWS but it just returns the number of spaces
pub fn fws_counted<'a>() -> impl Parser<'a, &'a str, usize, ErrType<'a>> {
    let rfc2822_fws = {
        let wsp_then_crlf = wsp().repeated().count().then_ignore(crlf()).or_not();
        wsp_then_crlf
            .then(wsp().repeated().at_least(1).count())
            .map(|(wsp_then_crlf, wsp)| {
                if let Some(mut wsp_then_crlf) = wsp_then_crlf {
                    wsp_then_crlf += wsp;
                    wsp_then_crlf
                } else {
                    wsp
                }
            })
    };
    // TODO Support obs_fws
    rfc2822_fws
}
pub fn quoted_pair<'a>() -> impl Parser<'a, &'a str, [char; 2], ErrType<'a>> {
    just('\\').ignore_then(text()).map(|v| ['\\', v])
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
pub fn cfws<'a>() -> impl Parser<'a, &'a str, Vec<char>, ErrType<'a>> {
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
pub fn atext_seg<'a, C>() -> impl Parser<'a, &'a str, C, ErrType<'a>>
where
    C: Container<char>,
{
    atext().repeated().at_least(1).collect::<C>()
}
/// An Atom
pub fn atom<'a>() -> impl Parser<'a, &'a str, Vec<char>, ErrType<'a>> {
    cfws()
        .or_not()
        .ignore_then(atext().repeated().at_least(1).collect::<Vec<char>>())
        .then_ignore(cfws().or_not())
}

/// ```ebnf
/// dot-atom-text = 1*atext *("." 1*atext)
/// ```
pub fn dot_atom_text<'a>() -> impl Parser<'a, &'a str, String, ErrType<'a>> {
    atext_seg::<String>()
        .separated_by(just('.'))
        .collect::<Vec<_>>()
        .map(|v| {
            if v.len() == 1 {
                return v.into_iter().next().unwrap();
            }
            let mut s = String::with_capacity(v.iter().map(|v| v.len() + 1).sum::<usize>());

            s.push_str(&v[0]);
            for v in v[1..].iter() {
                s.push('.');
                s.push_str(&v);
            }
            s
        })
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
pub fn qcontent<'a>() -> impl Parser<'a, &'a str, Either<char, [char; 2]>, ErrType<'a>> {
    qtext()
        .map(|v| Either::Left(v))
        .or(quoted_pair().map(|v| Either::Right(v)))
}
/// ```ebnf
/// quoted-string   =       [CFWS]
///                        DQUOTE *([FWS] qcontent) [FWS] DQUOTE
///                       [CFWS]
/// ```
pub fn quoted_string<'a>() -> impl Parser<'a, &'a str, Vec<char>, ErrType<'a>> {
    let inner = fws_counted()
        .or_not()
        .then(qcontent())
        .repeated()
        .collect::<Vec<_>>()
        .delimited_by(dquote(), fws().or_not().then(dquote()))
        .map(|v| {
            let mut s = Vec::with_capacity(v.len());
            for (spaces, c) in v {
                if let Some(spaces) = spaces {
                    for _ in 0..spaces {
                        s.push(' ');
                    }
                }
                match c {
                    Either::Left(c) => s.push(c),
                    Either::Right([escape, c]) => {
                        s.push(escape);
                        s.push(c);
                    }
                }
            }
            s
        });

    cfws()
        .or_not()
        .ignore_then(inner)
        .then_ignore(cfws().or_not())
}
pub fn word<'a>() -> impl Parser<'a, &'a str, Vec<char>, ErrType<'a>> {
    choice((quoted_string(), atom()))
}

pub fn obs_phrase<'a>() -> impl Parser<'a, &'a str, Vec<char>, ErrType<'a>> {
    let obs_phrase_part_two = choice((word(), just('.').map(|v| vec![v]), cfws()))
        .repeated()
        .collect::<Vec<_>>();
    word().then(obs_phrase_part_two).map(|(mut v, two)| {
        v.extend(two.into_iter().flatten());
        v
    })
}
pub fn pharse<'a>() -> impl Parser<'a, &'a str, Vec<char>, ErrType<'a>> {
    choice((word(), obs_phrase()))
}

pub fn dot_atom<'a>() -> impl Parser<'a, &'a str, String, ErrType<'a>> {
    cfws()
        .or_not()
        .ignore_then(dot_atom_text())
        .then_ignore(cfws().or_not())
}
pub fn local_part<'a>() -> impl Parser<'a, &'a str, String, ErrType<'a>> {
    choice((
        quoted_string().map(|v| {
            let mut s = String::with_capacity(v.len() + 2);
            s.push('"');
            for c in v {
                s.push(c);
            }
            s.push('"');
            s
        }),
        dot_atom(),
    ))
}
pub fn domain<'a>() -> impl Parser<'a, &'a str, String, ErrType<'a>> {
    choice((domain_literal(), dot_atom()))
}
pub fn dtext<'a>() -> impl Parser<'a, &'a str, char, ErrType<'a>> {
    choice((
        no_ws_ctl(),
        one_of('\x21'..='\x5A'),
        one_of('\x5E'..='\x7E'),
    ))
}
pub fn dcontent<'a>() -> impl Parser<'a, &'a str, Either<char, [char; 2]>, ErrType<'a>> {
    choice((
        dtext().map(|v| Either::Left(v)),
        quoted_pair().map(|v| Either::Right(v)),
    ))
}
pub fn domain_literal<'a>() -> impl Parser<'a, &'a str, String, ErrType<'a>> {
    let domain_inner = fws()
        .or_not()
        .ignore_then(dcontent())
        .repeated()
        .collect::<Vec<_>>()
        .map(|v| {
            let mut s = String::with_capacity(v.len() + 2);
            s.push('[');
            for c in v {
                match c {
                    Either::Left(c) => s.push(c),
                    Either::Right([escape, c]) => {
                        s.push(escape);
                        s.push(c);
                    }
                }
            }
            s.push(']');
            s
        });
    crlf()
        .or_not()
        .then(just('['))
        .ignore_then(domain_inner)
        .then_ignore(fws().or_not())
        .then_ignore(just(']'))
        .then_ignore(crlf().or_not())
}
pub fn addr_spec<'a>() -> impl Parser<'a, &'a str, (String, String), ErrType<'a>> {
    local_part().then_ignore(just('@')).then(domain())
}
pub fn angle_addr<'a>() -> impl Parser<'a, &'a str, (String, String), ErrType<'a>> {
    just('<').ignore_then(addr_spec()).then_ignore(just('>'))
}
pub fn display_name<'a>() -> impl Parser<'a, &'a str, String, ErrType<'a>> {
    pharse().map(|v| v.into_iter().collect())
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
#[cfg(test)]
mod tests {
    use chumsky::Parser;

    use crate::parsers::rfcs::rfc2822::{domain, domain_literal, quoted_string};

    use super::{display_name, fws_counted, mailbox};
    use pretty_assertions::assert_eq;
    #[test]
    pub fn test_mailbox() {
        let v = mailbox()
            .parse("\"Simple Test\" <test@mail.local>")
            .into_result();
        assert_eq!(
            v,
            Ok(super::RawMailBox {
                display_name: Some("Simple Test".to_string()),
                local: "test".to_string(),
                domain: "mail.local".to_string()
            })
        );
    }

    #[test]
    pub fn test_fws() {
        assert_eq!(fws_counted().parse("  ").into_result(), Ok(2))
    }

    #[test]
    pub fn test_display_name() {
        assert_eq!(
            display_name().parse("John").into_result(),
            Ok("John".to_string())
        );
        assert_eq!(
            display_name().parse(r#""Darth Vader""#).into_result(),
            Ok(r#"Darth Vader"#.to_string())
        );
    }
    #[test]
    pub fn test_quoted_string() {
        assert_eq!(
            quoted_string()
                .parse(r#""Darth Vader""#)
                .into_result()
                .map(|v| v.into_iter().collect::<String>()),
            Ok("Darth Vader".to_string())
        );
    }
    #[test]
    pub fn test_domain_literal() {
        assert_eq!(
            domain_literal().parse("[127.0.0.1]").into_result(),
            Ok("[127.0.0.1]".to_string())
        );
        assert_eq!(
            domain_literal()
                .parse("[2001:0db8:85a3:0000:0000:8a2e:0370:7334]")
                .into_result(),
            Ok("[2001:0db8:85a3:0000:0000:8a2e:0370:7334]".to_string())
        );
    }
    #[test]
    pub fn test_domain() {
        assert_eq!(
            domain().parse("example.com").into_result(),
            Ok("example.com".to_string())
        );
        assert_eq!(
            domain().parse("[127.0.0.1]").into_result(),
            Ok("[127.0.0.1]".to_string())
        );

        assert_eq!(
            domain()
                .parse("[2001:0db8:85a3:0000:0000:8a2e:0370:7334]")
                .into_result(),
            Ok("[2001:0db8:85a3:0000:0000:8a2e:0370:7334]".to_string())
        );
    }
}
#[cfg(test)]
mod address_spec_tests {
    use chumsky::Parser;
    use pretty_assertions::assert_eq;

    use crate::parsers::rfcs::rfc2822::addr_spec;

    fn check(raw: &str, local: &str, domain: &str) {
        let email_address = addr_spec().parse(raw).into_result();
        if let Err(e) = email_address {
            panic!("{:?}: {:?}", raw, e);
        }
        let email_address = email_address.unwrap();
        assert_eq!(email_address.0, local);
        assert_eq!(email_address.1, domain);
    }
    #[test]
    fn test_email_address() {
        check("email@example.com", "email", "example.com");
    }

    #[test]
    fn parse_a_bunch() {
        let instant = std::time::Instant::now();
        for _ in 0..1000 {
            let _ = addr_spec().parse("example@exmaple.com");
        }
        println!("1000 in {:?}", instant.elapsed());
    }
    #[test]
    fn test_weird_cases() {
        check(
            "disposable.style.email.with+symbol@example.com",
            "disposable.style.email.with+symbol",
            "example.com",
        );
        check(
            "other.email-with-dash@example.com",
            "other.email-with-dash",
            "example.com",
        );
        check("x@example.com", "x", "example.com");
        check(
            r#""much.more unusual"@example.com"#,
            r#""much.more unusual""#,
            "example.com",
        );
        check(
            r#""very.(),:;<>[]\".VERY.\"very@ \"very\".unusual"@strange.example.com"#,
            r#""very.(),:;<>[]\".VERY.\"very@ \"very\".unusual""#,
            "strange.example.com",
        );
    }
}
