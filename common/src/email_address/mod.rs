mod validation;

use std::fmt::Display;
use std::hash::{Hash, Hasher};
use std::ops::{Deref, DerefMut};
use std::str::FromStr;
use thiserror::Error;
pub use validation::{validate_domain, validate_user};
#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
#[error("Invalid Email Address: {0}")]
pub struct InvalidEmailAddress(&'static str);

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct EmailAddress {
    pub email_address: String,
    pub at_index: usize,
}
impl Deref for EmailAddress {
    type Target = String;
    fn deref(&self) -> &Self::Target {
        &self.email_address
    }
}
impl Hash for EmailAddress {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.email_address.hash(state);
    }
}

impl DerefMut for EmailAddress {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.email_address
    }
}
impl Display for EmailAddress {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.email_address.fmt(f)
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
    pub fn new(email_address: impl Into<String>) -> Result<Self, InvalidEmailAddress> {
        let email_address: String = email_address.into();
        let email_address_split = email_address.splitn(2, '@').collect::<Vec<_>>();
        if email_address_split.len() != 2 {
            return Err(InvalidEmailAddress("Missing @"));
        }
        let user = email_address_split[0];
        validate_user(user)?;
        let domain = email_address_split[1];
        validate_domain(domain)?;
        let at_index = email_address.find('@').unwrap();
        Ok(EmailAddress {
            email_address,
            at_index,
        })
    }
    pub fn new_only_check_for_at(
        email_address: impl Into<String>,
    ) -> Result<Self, InvalidEmailAddress> {
        let email_address: String = email_address.into();
        let at_index = email_address
            .find('@')
            .ok_or(InvalidEmailAddress("Missing @"))?;

        Ok(EmailAddress {
            email_address,
            at_index,
        })
    }
    pub fn new_unchecked(email_address: impl Into<String>) -> Self {
        let email_address: String = email_address.into();
        let at_index = email_address.find('@').unwrap();
        EmailAddress {
            email_address,
            at_index,
        }
    }
    pub fn get_user(&self) -> &str {
        &self.email_address[..self.at_index]
    }
    pub fn get_domain(&self) -> &str {
        &self.email_address[self.at_index + 1..]
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
impl AsRef<str> for EmailAddress {
    fn as_ref(&self) -> &str {
        &self.email_address
    }
}
impl AsRef<String> for EmailAddress {
    fn as_ref(&self) -> &String {
        &self.email_address
    }
}

#[cfg(feature = "serde")]
mod _serde {
    use crate::email_address::EmailAddress;
    use serde::{Serialize, Serializer};
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
            let address = EmailAddress::new_unchecked("email@example.com");
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
    #[test]
    fn test_email_address() {
        let email_address = super::EmailAddress::new("email@example.com").unwrap();
        assert_eq!(email_address.get_user(), "email");
        assert_eq!(email_address.get_domain(), "example.com");
    }
    fn test_email_unchecked() {
        let email_address = super::EmailAddress::new_unchecked("email@example.com");
        assert_eq!(email_address.get_user(), "email");
        assert_eq!(email_address.get_domain(), "example.com");
    }
    #[test]
    #[should_panic]
    fn test_email_address_panic() {
        let _ = super::EmailAddress::new("invalid{at}email.com").unwrap();
    }
}
