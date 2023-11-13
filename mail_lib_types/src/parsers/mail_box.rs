/*!
 * Parses a [Mailbox]
 * Based on the RFC 2822 spec: https://datatracker.ietf.org/doc/html/rfc2822
 * MailBoxes are in the following formats:
 * - `email@example.com`
 * - `"Display Name" <email@example.com>
 * - `"Display Name"<email@example.com>
 * - `<email@example.com>`
 * - `email@example.com`
 */

use chumsky::prelude::*;

use super::base;
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct RawMailBox {
    pub display_name: Option<String>,
    pub local: String,
    pub domain: String,
}

pub fn mailbox<'a>() -> impl Parser<'a, &'a str, RawMailBox, extra::Err<Cheap>> {
    choice((
        name_addr(),
        base::addr_spec().map(|addr| {
            let (local, domain) = addr;
            RawMailBox {
                display_name: None,
                local,
                domain,
            }
        }),
    ))
}
/// Name <email@example>
/// "Display Name" <email@example>
pub fn name_addr<'a>() -> impl Parser<'a, &'a str, RawMailBox, extra::Err<Cheap>> {
    base::pharse()
        .padded()
        .then_ignore(just('<'))
        .then(base::addr_spec())
        .then_ignore(just('>'))
        .map(|(display_name, (local, domain))| RawMailBox {
            display_name: Some(display_name),
            local,
            domain,
        })
}

#[cfg(test)]
pub mod tests {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn test_mailbox() {
        let parser = mailbox();
        let result = parser.parse("Test <username@example.com>").into_result();
        match result {
            Ok(ok) => {
                assert_eq!(
                    ok,
                    RawMailBox {
                        display_name: Some("Test".to_string()),
                        local: "username".to_string(),
                        domain: "example.com".to_string(),
                    }
                );
            }
            Err(err) => {
                for ele in err {
                    println!("{}", ele);
                }
                panic!();
            }
        }

        assert_eq!(
            parser
                .parse("\"Display Name\" <username@example.com>")
                .into_result(),
            Ok(RawMailBox {
                display_name: Some("Display Name".to_string()),
                local: "username".to_string(),
                domain: "example.com".to_string(),
            })
        );
    }

    #[test]
    fn test_name_addr() {
        let parser = name_addr();

        assert_eq!(
            parser
                .parse("\"Display Name\"<username@example.com>")
                .unwrap(),
            RawMailBox {
                display_name: Some("Display Name".to_string()),
                local: "username".to_string(),
                domain: "example.com".to_string(),
            }
        );
    }
}
