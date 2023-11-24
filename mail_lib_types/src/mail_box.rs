/*!
#  MailBox

A [MailBox] is a structure that contains a [EmailAddress] and an optional name.

Defined in [RFC 5322 Section 3.4](https://tools.ietf.org/html/rfc5322#section-3.4)
 */
use std::{borrow::Cow, fmt::Display, str::FromStr};

use chumsky::{error::Cheap, Parser};
use digestible::Digestible;
use thiserror::Error;

use crate::{parsers::rfcs::rfc2822::mailbox, EmailAddress};
/// Used Internally as a temporary structure to build a [MailBox]
#[doc(hidden)]
#[derive(Debug, PartialEq, Eq)]
pub struct RawMailBox {
    pub(crate) display_name: Option<String>,
    pub(crate) local: String,
    pub(crate) domain: String,
}
impl FromStr for RawMailBox {
    type Err = InvalidMailBox<'static>;
    fn from_str(value: &str) -> Result<Self, Self::Err> {
        let parsed = mailbox().parse(value).into_result();
        match parsed {
            Ok(v) => Ok(v.into()),
            Err(e) => Err(InvalidMailBox {
                spans: e,
                ctx: None,
            }),
        }
    }
}
impl TryFrom<String> for RawMailBox {
    type Error = InvalidMailBox<'static>;
    fn try_from(value: String) -> Result<Self, Self::Error> {
        let parsed = mailbox().parse(value.as_str()).into_result();
        match parsed {
            Ok(v) => Ok(v.into()),
            Err(e) => Err(InvalidMailBox {
                spans: e,
                ctx: Some(Cow::Owned(value)),
            }),
        }
    }
}
impl<'a> TryFrom<&'a str> for RawMailBox {
    type Error = InvalidMailBox<'a>;
    fn try_from(value: &'a str) -> Result<Self, Self::Error> {
        let parsed = mailbox().parse(value).into_result();
        match parsed {
            Ok(v) => Ok(v.into()),
            Err(e) => Err(InvalidMailBox {
                spans: e,
                ctx: Some(Cow::Borrowed(value)),
            }),
        }
    }
}

impl Into<(Option<String>, String, String)> for RawMailBox {
    fn into(self) -> (Option<String>, String, String) {
        (self.display_name, self.local, self.domain)
    }
}
/// A [MailBox] is a structure that contains a [EmailAddress] and an optional name.
#[derive(Debug, Clone, PartialOrd, Ord, PartialEq, Eq, Hash, Digestible)]
pub struct MailBox {
    /// The optional name of the mailbox
    pub name: Option<String>,
    /// The email address of the mailbox
    pub email: EmailAddress,
}
impl Display for MailBox {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.name.as_deref() {
            Some(name) => write!(f, "{} <{}>", name, self.email),
            None => write!(f, "{}", self.email),
        }
    }
}
impl MailBox {
    /// Create a new [MailBox] with the given name and email address
    pub fn new(name: Option<String>, email: EmailAddress) -> Self {
        Self { name, email }
    }
    /// Get the local part of the email address
    pub fn get_local(&self) -> &str {
        self.email.get_local()
    }
    /// Get the domain part of the email address
    pub fn get_domain(&self) -> &str {
        self.email.get_domain()
    }
    /// Get the name of the mailbox
    pub fn get_name(&self) -> Option<&str> {
        self.name.as_deref()
    }
    /// Convert the [MailBox] into its parts
    pub fn into_inner(self) -> (Option<String>, EmailAddress) {
        (self.name, self.email)
    }
}
#[cfg(feature = "serde")]
mod _serde {
    use serde::{ser::SerializeStruct, Deserialize, Serialize};

    use super::MailBox;
    use crate::EmailAddress;
    /// Serialize a [MailBox] as an object with the fields `name` and `email`
    pub fn serialize_as_object<S>(mailbox: &MailBox, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut ser = serializer.serialize_struct("MailBox", 2)?;
        if let Some(name) = mailbox.name.as_ref() {
            ser.serialize_field("name", name)?;
        }
        ser.serialize_field("email", &mailbox.email)?;
        ser.end()
    }

    impl Serialize for MailBox {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: serde::Serializer,
        {
            match &self.name {
                Some(name) => serializer.serialize_str(&format!("{} <{}>", name, self.email)),
                None => serializer.serialize_str(self.email.as_ref()),
            }
        }
    }
    struct MailBoxVisitor;

    impl<'de> serde::de::Visitor<'de> for MailBoxVisitor {
        type Value = MailBox;

        fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
            formatter.write_str("a valid email address")
        }

        fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
        where
            E: serde::de::Error,
        {
            MailBox::try_from(value).map_err(serde::de::Error::custom)
        }

        fn visit_string<E>(self, v: String) -> Result<Self::Value, E>
        where
            E: serde::de::Error,
        {
            MailBox::try_from(v).map_err(serde::de::Error::custom)
        }

        fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
        where
            A: serde::de::MapAccess<'de>,
        {
            let mut name: Option<String> = None;
            let mut email: Option<String> = None;
            while let Some(key) = map.next_key::<String>()? {
                match key.as_str() {
                    "name" => {
                        if name.is_some() {
                            return Err(serde::de::Error::duplicate_field("name"));
                        }
                        name = Some(map.next_value()?);
                    }
                    "email" => {
                        if email.is_some() {
                            return Err(serde::de::Error::duplicate_field("email"));
                        }
                        email = Some(map.next_value()?);
                    }
                    _ => {
                        return Err(serde::de::Error::unknown_field(
                            key.as_str(),
                            &["name", "email"],
                        ))
                    }
                }
            }
            let email = email.ok_or_else(|| serde::de::Error::missing_field("email"))?;
            let address = EmailAddress::new(email.as_str()).map_err(serde::de::Error::custom)?;
            Ok(MailBox::new(name, address))
        }
    }
    impl<'de> Deserialize<'de> for MailBox {
        fn deserialize<D>(deserializer: D) -> Result<MailBox, D::Error>
        where
            D: serde::Deserializer<'de>,
        {
            deserializer.deserialize_any(MailBoxVisitor)
        }
    }
}
#[cfg(feature = "serde")]
#[doc(inline)]
pub use _serde::serialize_as_object;
/// An error that occurs when parsing a [MailBox]
#[derive(Debug, Clone, PartialEq, Hash, Error)]
pub struct InvalidMailBox<'a> {
    /// The spans that caused the error
    pub spans: Vec<Cheap>,
    /// The context of the error
    pub ctx: Option<Cow<'a, str>>,
}
impl Display for InvalidMailBox<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Invalid MailBox")?;
        if let Some(context) = self.ctx.as_ref() {
            writeln!(f, "Context: {}", context)?;
            for span in &self.spans {
                writeln!(f, "    {}", span)?;
            }
        }
        Ok(())
    }
}
impl From<RawMailBox> for MailBox {
    fn from(value: RawMailBox) -> Self {
        let RawMailBox {
            display_name,
            local,
            domain,
        } = value;
        // Safe as long as the parser did its job
        let email = unsafe { EmailAddress::new_unchecked_from_parts(local, domain) };
        MailBox::new(display_name, email)
    }
}
impl PartialEq<RawMailBox> for MailBox {
    fn eq(&self, other: &RawMailBox) -> bool {
        self.name == other.display_name
            && self.email.get_local() == other.local
            && self.email.get_domain() == other.domain
    }
}
impl FromStr for MailBox {
    type Err = InvalidMailBox<'static>;
    fn from_str(value: &str) -> Result<Self, Self::Err> {
        RawMailBox::from_str(value).map(MailBox::from)
    }
}
impl TryFrom<String> for MailBox {
    type Error = InvalidMailBox<'static>;
    fn try_from(value: String) -> Result<Self, Self::Error> {
        RawMailBox::try_from(value).map(MailBox::from)
    }
}
impl<'a> TryFrom<&'a String> for MailBox {
    type Error = InvalidMailBox<'a>;
    fn try_from(value: &'a String) -> Result<Self, Self::Error> {
        RawMailBox::try_from(value.as_str()).map(MailBox::from)
    }
}
impl<'a> TryFrom<&'a str> for MailBox {
    type Error = InvalidMailBox<'a>;
    fn try_from(value: &'a str) -> Result<Self, Self::Error> {
        RawMailBox::try_from(value).map(MailBox::from)
    }
}
