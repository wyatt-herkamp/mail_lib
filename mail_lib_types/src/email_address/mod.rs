/*!
# Representation of an Email Address
 */
mod validation;

use std::{
    fmt::{Debug, Display},
    hash::{Hash, Hasher},
    ops::Deref,
    str::FromStr,
};

use chumsky::Parser;
use digestible::Digestible;
use strum::Display;
pub use validation::{
    validate_domain, validate_local_part, EmailErrorMessage, InvalidEmailAddress,
};

/// Email Address Parts
#[derive(Debug, Clone, Copy, PartialEq, Eq, Display)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum EmailPart {
    /// Local Part of email address.
    ///
    /// Before the @
    Local,
    /// Domain Part of email address.
    ///
    /// After the @
    Domain,
}
pub use mail_lib_macros::from_static_email;

use crate::parsers::rfcs::rfc5322;
/// Representation of an Email Address
///
/// Fields are private to prevent invalid email addresses from being created.
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct EmailAddress {
    email_address: String,
    at_index: usize,
}
impl Debug for EmailAddress {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("EmailAddress")
            .field("local_part", &self.get_local())
            .field("domain", &self.get_domain())
            .finish()
    }
}
impl Deref for EmailAddress {
    type Target = str;
    fn deref(&self) -> &Self::Target {
        &self.email_address
    }
}
impl AsRef<str> for EmailAddress {
    fn as_ref(&self) -> &str {
        &self.email_address
    }
}
impl Digestible for EmailAddress {
    fn digest<B: digestible::byteorder::ByteOrder, W: digestible::DigestWriter>(
        &self,
        writer: &mut W,
    ) {
        self.email_address.digest::<B, W>(writer)
    }
}
impl Hash for EmailAddress {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.email_address.hash(state);
    }
}

impl Display for EmailAddress {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Display::fmt(&self.email_address, f)
    }
}

impl PartialEq<String> for EmailAddress {
    fn eq(&self, other: &String) -> bool {
        self.email_address == *other
    }
}
impl PartialEq<&str> for EmailAddress {
    fn eq(&self, other: &&str) -> bool {
        self.email_address == *other
    }
}
impl EmailAddress {
    /// Creates a new Email Address.
    ///
    /// Checks for Validity
    pub fn new(email_address: impl Into<String>) -> Result<Self, InvalidEmailAddress> {
        let email_address: String = email_address.into();
        let Some((local, domain)) = email_address.rsplit_once('@') else {
            return Err(InvalidEmailAddress::from("Missing @"));
        };
        // This Logging is only for Testing
        #[cfg(all(feature = "tracing", debug_assertions))]
        tracing::trace!("Email Address: {:?} at {:?}", local, domain);
        validate_local_part(local)?;
        validate_domain(domain)?;
        let at_index = local.len();
        Ok(EmailAddress {
            email_address,
            at_index,
        })
    }
    /// This only exists for Benchmarking and Testing.
    ///
    /// The pure Rust Implementation is faster at validating email addresses.
    #[doc(hidden)]
    pub fn new_validate_over_chumsky(
        email_address: impl AsRef<str>,
    ) -> Result<Self, InvalidEmailAddress> {
        let result = rfc5322::addr_spec()
            .parse(email_address.as_ref())
            .into_result();
        if let Ok((local, _)) = result {
            Ok(EmailAddress {
                email_address: email_address.as_ref().to_owned(),
                at_index: local.len(),
            })
        } else {
            Err(InvalidEmailAddress::from("Unknown Error"))
        }
    }
    /// Creates a new Email Address.
    ///
    /// Only CHecks for the `@` Symbol. This will not cause any undefined behavior however, could lead to command injection.
    pub unsafe fn new_only_check_for_at(
        email_address: impl Into<String>,
    ) -> Result<Self, InvalidEmailAddress> {
        let email_address: String = email_address.into();
        let at_index = email_address
            .rfind('@')
            .ok_or(InvalidEmailAddress::from("Missing @"))?;

        Ok(EmailAddress {
            email_address,
            at_index,
        })
    }
    /// Creates an email address without checking for validity.
    /// Takes a String and the index of the `@` symbol.
    /// This is unsafe because it could cause undefined behavior if the email is invalid.
    ///
    /// However, this is good for static strings that are known to be valid.
    pub unsafe fn new_unchecked_raw(email_address: impl Into<String>, at_index: usize) -> Self {
        EmailAddress {
            email_address: email_address.into(),
            at_index,
        }
    }
    /// Creates an email address without checking for validity.
    pub unsafe fn new_unchecked_from_parts(
        local: impl AsRef<str>,
        domain: impl AsRef<str>,
    ) -> Self {
        let local = local.as_ref();
        EmailAddress {
            email_address: combine_parts(local, domain.as_ref()),
            at_index: local.len(),
        }
    }
    /// Gets a reference to the Email Address parts
    pub fn as_parts(&self) -> (&str, &str) {
        self.email_address.split_at(self.at_index)
    }
    /// Gets the parts of the Email Address as owned Strings
    pub fn parts_owned(&self) -> (String, String) {
        let (local, domain) = self.email_address.split_at(self.at_index);
        (local.to_string(), domain.to_string())
    }
    /// Gets the Local Part of the Email Address
    #[inline]
    pub fn get_local(&self) -> &str {
        &self.email_address[..self.at_index]
    }
    /// Gets a reference to the Domain
    #[inline]
    pub fn get_domain(&self) -> &str {
        &self.email_address[self.at_index + 1..]
    }
    /// Gets a reference to the Email Address
    ///
    /// Deref and AsRef are also both implemented for EmailAddress
    pub fn as_str(&self) -> &str {
        &self.email_address
    }
}
impl Into<(String, String)> for EmailAddress {
    fn into(self) -> (String, String) {
        self.parts_owned()
    }
}
fn combine_parts(local: &str, domain: &str) -> String {
    let mut email_address = String::with_capacity(local.len() + domain.len() + 1);
    email_address.push_str(local);
    email_address.push('@');
    email_address.push_str(domain);
    email_address
}
impl<L: AsRef<str>, D: AsRef<str>> TryFrom<(L, D)> for EmailAddress {
    type Error = InvalidEmailAddress;

    fn try_from((local, domain): (L, D)) -> Result<Self, Self::Error> {
        validate_local_part(local.as_ref())?;
        validate_domain(local.as_ref())?;
        Ok(EmailAddress {
            email_address: combine_parts(local.as_ref(), domain.as_ref()),
            at_index: local.as_ref().len(),
        })
    }
}

impl FromStr for EmailAddress {
    type Err = InvalidEmailAddress;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        EmailAddress::new(s)
    }
}

impl TryFrom<String> for EmailAddress {
    type Error = InvalidEmailAddress;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        EmailAddress::new(value)
    }
}

impl Into<String> for EmailAddress {
    fn into(self) -> String {
        self.email_address
    }
}
impl AsRef<String> for EmailAddress {
    fn as_ref(&self) -> &String {
        &self.email_address
    }
}
#[cfg(feature = "rkyv")]
mod _rkyv {
    // TODO: Implement rkyv
}
#[cfg(feature = "serde")]
mod _serde {
    use serde::{Serialize, Serializer};

    use crate::email_address::EmailAddress;
    impl Serialize for EmailAddress {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
        {
            serializer.serialize_str(&self.email_address)
        }
    }
    impl<'de> serde::Deserialize<'de> for EmailAddress {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: serde::Deserializer<'de>,
        {
            let s = String::deserialize(deserializer)?;
            EmailAddress::new(s).map_err(serde::de::Error::custom)
        }
    }
    #[cfg(test)]
    mod tests {
        use crate::email_address::EmailAddress;

        #[derive(serde::Serialize, serde::Deserialize)]
        struct TestStruct {
            email_address: EmailAddress,
        }
        #[test]
        fn test_serde() {
            let address =
                unsafe { EmailAddress::new_only_check_for_at("email@example.com").unwrap() };
            let test_struct = TestStruct {
                email_address: address.clone(),
            };

            let json = serde_json::to_string(&test_struct).unwrap();
            assert!(json.contains("email@example.com"));
            let test_struct: TestStruct = serde_json::from_str(&json).unwrap();
            assert_eq!(test_struct.email_address, address)
        }
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::{from_static_email, EmailAddress};
    use crate::email_address::EmailErrorMessage;
    #[test]
    fn test_macro() {
        let email_address = from_static_email!("test@example.com");
        let safe_email_address = EmailAddress::new("test@example.com").unwrap();
        assert_eq!(email_address, safe_email_address);
    }

    fn check(raw: &str, local: &str, domain: &str) {
        let email_address = super::EmailAddress::new(raw);
        if let Err(e) = email_address {
            if let EmailErrorMessage::InvalidCharacter(c, pos) = e.error_message {
                println!("{}", raw);
                println!("{}^", " ".repeat(pos));
                panic!("Invalid character `{}` at position {}", c, pos)
            }
            panic!("{:?}: {}", raw, e);
        }
        let email_address = email_address.unwrap();
        assert_eq!(email_address.get_local(), local);
        assert_eq!(email_address.get_domain(), domain);
    }
    #[test]
    fn test_email_address() {
        let email_address = super::EmailAddress::new("email@example.com").unwrap();
        assert_eq!(email_address.get_local(), "email");
        assert_eq!(email_address.get_domain(), "example.com");
    }
    #[test]
    fn test_email_unchecked() {
        let email_address =
            unsafe { super::EmailAddress::new_only_check_for_at("email@example.com").unwrap() };
        assert_eq!(email_address.get_local(), "email");
        assert_eq!(email_address.get_domain(), "example.com");
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

    #[test]
    fn test_email_address_panic() {
        let v = super::EmailAddress::new("invalid{at}email.com");
        assert!(v.is_err())
    }
}
