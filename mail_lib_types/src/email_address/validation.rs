/*!
# Email Address Validation
Email Addresses are really weird. If you allow the wrong data to be entered you now have command injection. However, email addresses allow a lot of different characters.


This parser is based on [rust-email_address](https://github.com/johnstonskj/rust-email_address)

 */
use std::fmt::Display;

use thiserror::Error;

use super::EmailPart;

const SP: char = ' ';
const HTAB: char = '\t';
const ESC: char = '\\';

const DOT: char = '.';
const LBRACKET: char = '[';
const RBRACKET: char = ']';

const UTF8_START: char = '\u{0080}';
#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum EmailErrorMessage {
    #[error("Invalid Character: `{0}` at position {1}")]
    InvalidCharacter(char, usize),
    #[error("Invalid Length of Sub Domain(Valid Range is [1,63])")]
    InvalidLengthOfSubDomain,
    #[error("Invalid Length of User(Valid Range is [1,64])")]
    InvalidLengthOfUser,
    #[error("Invalid Length of Domain(Valid Range is [1,255])")]
    InvalidLengthOfDomain,
    #[error("{0}")]
    Other(&'static str),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub struct InvalidEmailAddress {
    pub error_part: Option<EmailPart>,
    pub error_message: EmailErrorMessage,
}
impl From<&'static str> for InvalidEmailAddress {
    fn from(error_message: &'static str) -> Self {
        InvalidEmailAddress {
            error_part: None,
            error_message: EmailErrorMessage::Other(error_message),
        }
    }
}
impl From<(EmailErrorMessage, EmailPart)> for InvalidEmailAddress {
    fn from((error_message, error_part): (EmailErrorMessage, EmailPart)) -> Self {
        InvalidEmailAddress {
            error_part: Some(error_part),
            error_message,
        }
    }
}

impl From<(&'static str, EmailPart)> for InvalidEmailAddress {
    fn from((error_message, error_part): (&'static str, EmailPart)) -> Self {
        InvalidEmailAddress {
            error_part: Some(error_part),
            error_message: EmailErrorMessage::Other(error_message),
        }
    }
}

impl Display for InvalidEmailAddress {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(error_part) = self.error_part {
            write!(f, "Invalid Email In {}: {}", error_part, self.error_message)
        } else {
            write!(f, "Invalid Email{}", self.error_message)
        }
    }
}
/// [Source](https://datatracker.ietf.org/doc/html/rfc5321#section-4.5.3.1.2)
pub static MAX_DOMAIN_LENGTH: usize = 255;
/// [Source](https://datatracker.ietf.org/doc/html/rfc5321#section-4.5.3.1.1)
pub static MAX_LOCAL_PART: usize = 64;
pub static SUB_DOMAIN_MAX_LENGTH: usize = 63;
static QUOTE: char = '"';

pub fn validate_domain(domain: &str) -> Result<(), InvalidEmailAddress> {
    if domain.is_empty() || domain.len() > MAX_DOMAIN_LENGTH {
        Err((EmailErrorMessage::InvalidLengthOfDomain, EmailPart::Domain).into())
    } else if domain.starts_with(LBRACKET) && domain.ends_with(RBRACKET) {
        parse_literal_domain(&domain[1..domain.len() - 1])
    } else {
        parse_text_domain(domain)
    }
}

pub fn validate_local_part(local: &str) -> Result<(), InvalidEmailAddress> {
    if local.is_empty() || local.len() > MAX_LOCAL_PART {
        Err((EmailErrorMessage::InvalidLengthOfUser, EmailPart::Local).into())
    } else if local.starts_with(QUOTE) && local.ends_with(QUOTE) {
        if local.len() == 2 {
            Err((EmailErrorMessage::InvalidLengthOfUser, EmailPart::Local).into())
        } else {
            parse_quoted_local_part(local)
        }
    } else {
        parse_unquoted_local_part(local)
    }
}

fn parse_text_domain(part: &str) -> Result<(), InvalidEmailAddress> {
    if let Some((c, pos)) = is_dot_atom_text(part) {
        return Err((
            EmailErrorMessage::InvalidCharacter(c, pos),
            EmailPart::Domain,
        )
            .into());
    }

    for sub_part in part.split(DOT) {
        if sub_part.len() > SUB_DOMAIN_MAX_LENGTH {
            return Err((
                EmailErrorMessage::InvalidLengthOfSubDomain,
                EmailPart::Domain,
            )
                .into());
        }
    }
    return Ok(());
}
fn parse_literal_domain(part: &str) -> Result<(), InvalidEmailAddress> {
    if part.chars().all(is_dtext_char) {
        return Ok(());
    }
    return Err(("Invalid Literal Domain", EmailPart::Domain).into());
}
fn is_dtext_char(c: char) -> bool {
    ('\x21'..='\x5A').contains(&c) || ('\x5E'..='\x7E').contains(&c)
}

fn parse_unquoted_local_part(part: &str) -> Result<(), InvalidEmailAddress> {
    if let Some((c, pos)) = is_dot_atom_text(part) {
        return Err((
            EmailErrorMessage::InvalidCharacter(c, pos),
            EmailPart::Local,
        )
            .into());
    }
    Ok(())
}

fn is_atext(c: char) -> bool {
    c.is_alphanumeric()
        || c == '!'
        || c == '#'
        || c == '$'
        || c == '%'
        || c == '&'
        || c == '\''
        || c == '*'
        || c == '+'
        || c == '-'
        || c == '/'
        || c == '='
        || c == '?'
        || c == '^'
        || c == '_'
        || c == '`'
        || c == '{'
        || c == '|'
        || c == '}'
        || c == '~'
        || is_uchar(c)
}

fn is_uchar(c: char) -> bool {
    c >= UTF8_START
}

fn is_dot_atom_text(s: &str) -> Option<(char, usize)> {
    let mut char_iter = s.char_indices();
    while let Some((pos, c)) = char_iter.next() {
        if c == DOT {
            // dot-atom-text
            match char_iter.next() {
                Some((_, c2)) if is_atext(c2) => (),
                _ => return Some((c, pos)),
            }
        } else if !is_atext(c) {
            // atom
            return Some((c, pos));
        }
    }
    None
}

fn is_vchar(c: char) -> bool {
    ('\x21'..='\x7E').contains(&c)
}

fn is_wsp(c: char) -> bool {
    c == SP || c == HTAB
}

fn is_qtext_char(c: char) -> bool {
    c == '\x21' || ('\x23'..='\x5B').contains(&c) || ('\x5D'..='\x7E').contains(&c) || is_uchar(c)
}
fn is_qcontent(s: &str) -> Option<(char, usize)> {
    let mut char_iter = s.char_indices();
    while let Some((pos, c)) = char_iter.next() {
        if c == ESC {
            // quoted-pair
            match char_iter.next() {
                Some((_, c2)) if is_vchar(c2) => (),
                _ => return Some((c, pos)),
            }
        } else if !(is_wsp(c) || is_qtext_char(c)) {
            // qtext
            return Some((c, pos));
        }
    }
    None
}

fn parse_quoted_local_part(part: &str) -> Result<(), InvalidEmailAddress> {
    let part = &part[1..part.len() - 1];
    println!("part: {}", part);
    if let Some((c, pos)) = is_qcontent(part) {
        return Err((
            EmailErrorMessage::InvalidCharacter(c, pos),
            EmailPart::Local,
        )
            .into());
    }
    Ok(())
}
