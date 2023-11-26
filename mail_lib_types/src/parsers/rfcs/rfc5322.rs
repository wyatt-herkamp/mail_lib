use chumsky::prelude::*;

use super::rfc5234::*;
use crate::{mail_box::RawMailBox, parsers::ErrType};

///
/// ```ebnf
/// qtext           =
///                        %d33 /          ; The rest of the US-ASCII
///                        %d35-91 /       ;  characters not including "\"
///                        %d93-126        ;  or the quote character
/// ```
pub fn qtext<'a>() -> impl Parser<'a, &'a str, &'a str, ErrType<'a>> {
    choice((
        one_of('\x5D'..='\x7E'), // ASCII 93-126
        one_of('\x23'..='\x5B'), // ASCII 35-91
        just('\x21'),            // ASCII 33
    ))
    .to_slice()
}
/// ```ebnf
/// quoted-pair     =       ("\" (VCHAR / WSP)) / obs-qp
/// ```
pub fn quoted_pair<'a>() -> impl Parser<'a, &'a str, &'a str, ErrType<'a>> {
    just('\\').then(vchar()).to_slice()
}
/// ```ebnf
/// qcontent        =       qtext / quoted-pair
/// ```
pub fn qcontent<'a>() -> impl Parser<'a, &'a str, &'a str, ErrType<'a>> {
    // Merge Choices instead of choice((qtext(), quoted_pair()))
    choice((
        one_of('\x5D'..='\x7E').to_slice(),  // ASCII 93-126
        one_of('\x23'..='\x5B').to_slice(),  // ASCII 35-91
        just('\x21').to_slice(),             // ASCII 33
        just('\\').then(vchar()).to_slice(), // `\` followed by a vchar (visible character)
    ))
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
/// Same as [`quoted_string`] but will strip the quotes
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
// These are the same as in rfc2822
#[doc(inline)]
pub use super::rfc2822::{
    atext, atom, ccontent, cfws, comment, ctext, dot_atom, dot_atom_text, fws,
};
/// ```ebnf
/// word            =       atom / quoted-string
/// ```
pub fn word<'a>() -> impl Parser<'a, &'a str, &'a str, ErrType<'a>> {
    choice((atom(), quoted_string()))
}
/// Same as [`word`] but will strip the quotes
pub fn word_strip_quotes<'a>() -> impl Parser<'a, &'a str, &'a str, ErrType<'a>> {
    choice((atom(), quoted_string_strip_quotes()))
}
/// ```ebnf
///    phrase          =   1*word / obs-phrase
/// ```
pub fn pharse<'a>() -> impl Parser<'a, &'a str, &'a str, ErrType<'a>> {
    // TODO: obs-phrase
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

/// ```ebnf
/// local-part      =       dot-atom / quoted-string / obs-local-part
/// ```
pub fn local_part<'a>() -> impl Parser<'a, &'a str, &'a str, ErrType<'a>> {
    choice((quoted_string(), dot_atom()))
}
/// ```ebnf
/// domain          =       dot-atom / domain-literal / obs-domain
/// ```
pub fn domain<'a>() -> impl Parser<'a, &'a str, &'a str, ErrType<'a>> {
    choice((domain_literal(), dot_atom()))
}
/// ```ebnf
/// dtext           =       %d33-90 /          ; Printable US-ASCII
///                        %d94-126 /         ;  characters not including
///                       obs-dtext          ;  "[", "]", or "\"
/// ```
pub fn dtext<'a>() -> impl Parser<'a, &'a str, &'a str, ErrType<'a>> {
    choice((one_of('\x21'..='\x5A'), one_of('\x5E'..='\x7E'))).to_slice()
}

/// ```ebnf
/// domain-literal  =       [CFWS] "[" *([FWS] dcontent) [FWS] "]" [CFWS]
/// ```
pub fn domain_literal<'a>() -> impl Parser<'a, &'a str, &'a str, ErrType<'a>> {
    let domain_inner = fws().or_not().ignore_then(dtext()).repeated().to_slice();
    crlf()
        .or_not()
        .ignore_then(just('['))
        .then(domain_inner)
        .then_ignore(fws().or_not())
        .then(just(']'))
        .then_ignore(crlf().or_not())
        .to_slice()
}
/// ```ebnf
/// addr-spec       =       local-part "@" domain
/// ```
pub fn addr_spec<'a>() -> impl Parser<'a, &'a str, (&'a str, &'a str), ErrType<'a>> {
    local_part().then_ignore(just('@')).then(domain())
}
/// ```ebnf
/// angle-addr      =       [CFWS] "<" addr-spec ">" [CFWS] / obs-angle-addr
/// ```
pub fn angle_addr<'a>() -> impl Parser<'a, &'a str, (&'a str, &'a str), ErrType<'a>> {
    just('<').ignore_then(addr_spec()).then_ignore(just('>'))
}

pub fn name_addr<'a>() -> impl Parser<'a, &'a str, RawMailBox<'a>, ErrType<'a>> {
    display_name()
        .or_not()
        .padded()
        .then(angle_addr())
        .map(|(display_name, (local, domain))| RawMailBox::new(display_name, local, domain))
}

pub fn mailbox<'a>() -> impl Parser<'a, &'a str, RawMailBox<'a>, ErrType<'a>> {
    choice((
        name_addr(),
        addr_spec().map(|(local, domain)| RawMailBox::new_no_name(local, domain)),
    ))
}

#[cfg(test)]
mod tests {
    use chumsky::Parser;
    use pretty_assertions::assert_eq;

    use super::{display_name, mailbox};
    use super::{domain, domain_literal, quoted_string};
    #[test]
    pub fn test_mailbox() {
        let v = mailbox()
            .parse("\"Simple Test\" <test@mail.local>")
            .into_result();
        assert_eq!(
            v,
            Ok(super::RawMailBox {
                display_name: Some("Simple Test".into()),
                local: "test".into(),
                domain: "mail.local".into()
            })
        );
    }

    #[test]
    pub fn test_display_name() {
        assert_eq!(display_name().parse("John").into_result(), Ok("John"));
        assert_eq!(
            display_name().parse(r#""Darth Vader""#).into_result(),
            Ok(r#"Darth Vader"#)
        );
    }
    #[test]
    pub fn test_quoted_string() {
        assert_eq!(
            quoted_string().parse(r#""Darth Vader""#).into_result(),
            Ok(r#""Darth Vader""#)
        );
    }
    #[test]
    pub fn test_domain_literal() {
        assert_eq!(
            domain_literal().parse("[127.0.0.1]").into_result(),
            Ok("[127.0.0.1]")
        );
        assert_eq!(
            domain_literal()
                .parse("[2001:0db8:85a3:0000:0000:8a2e:0370:7334]")
                .into_result(),
            Ok("[2001:0db8:85a3:0000:0000:8a2e:0370:7334]")
        );
    }
    #[test]
    pub fn test_domain() {
        assert_eq!(
            domain().parse("example.com").into_result(),
            Ok("example.com")
        );
        assert_eq!(
            domain().parse("[127.0.0.1]").into_result(),
            Ok("[127.0.0.1]")
        );

        assert_eq!(
            domain()
                .parse("[2001:0db8:85a3:0000:0000:8a2e:0370:7334]")
                .into_result(),
            Ok("[2001:0db8:85a3:0000:0000:8a2e:0370:7334]")
        );
    }
}
#[cfg(test)]
mod address_spec_tests {
    use chumsky::Parser;
    use pretty_assertions::assert_eq;

    use super::addr_spec;

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
