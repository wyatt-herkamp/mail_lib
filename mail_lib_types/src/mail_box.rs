use std::fmt::Display;

use chumsky::{error::Cheap, Parser};
use thiserror::Error;

use crate::{parsers::mail_box::RawMailBox, EmailAddress};

#[derive(Debug, Clone, PartialOrd, Ord, PartialEq, Eq, Hash)]
pub struct Mailbox {
    pub name: Option<String>,
    pub email: EmailAddress,
}
impl Display for Mailbox {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.name {
            Some(name) => write!(f, "{} <{}>", name, self.email),
            None => write!(f, "{}", self.email),
        }
    }
}
impl Mailbox {
    pub fn new(name: Option<String>, email: EmailAddress) -> Self {
        Self { name, email }
    }
    pub fn parse<'a>(input: &'a str) -> Result<Self, InvalidMailBox<'a>> {
        Mailbox::try_from(input)
    }
    pub fn get_local(&self) -> &str {
        self.email.get_local()
    }
    pub fn get_domain(&self) -> &str {
        self.email.get_domain()
    }
    pub fn get_name(&self) -> Option<&str> {
        self.name.as_deref()
    }
}
#[cfg(feature = "serde")]
mod _serde {
    use serde::{Deserialize, Serialize};

    use super::Mailbox;

    impl Serialize for Mailbox {
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
    impl<'de> Deserialize<'de> for Mailbox {
        fn deserialize<D>(deserializer: D) -> Result<Mailbox, D::Error>
        where
            D: serde::Deserializer<'de>,
        {
            let s = String::deserialize(deserializer)?;
            Ok(Mailbox::try_from(s.as_str()).map_err(serde::de::Error::custom)?)
        }
    }
}
#[derive(Debug, Clone, PartialEq, Hash, Error)]
pub struct InvalidMailBox<'a> {
    pub spans: Vec<Cheap>,
    pub ctx: Option<&'a str>,
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
impl<'a> TryFrom<&'a str> for Mailbox {
    type Error = InvalidMailBox<'a>;
    fn try_from(value: &'a str) -> Result<Self, Self::Error> {
        let parsed = crate::parsers::mail_box::mailbox()
            .parse(value)
            .into_result();
        match parsed {
            Ok(v) => {
                let RawMailBox {
                    display_name,
                    local,
                    domain,
                } = v;
                let email = unsafe { EmailAddress::new_unchecked_from_parts(local, domain) };
                Ok(Mailbox::new(display_name, email))
            }
            Err(e) => Err(InvalidMailBox {
                spans: e,
                ctx: Some(value),
            }),
        }
    }
}
