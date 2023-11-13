use base64::engine::general_purpose::STANDARD;
use base64::Engine;
use enum_helper::{EnumOfKeys, HasKeyEnum};
use std::fmt::{Debug, Display};
use std::string::FromUtf8Error;
use strum::{AsRefStr, Display, EnumIs, EnumString, IntoStaticStr};
use thiserror::Error;

#[derive(Debug, Error)]
#[non_exhaustive]
pub enum CredentialError {
    #[error("Invalid base64: {0}")]
    Base64Error(#[from] base64::DecodeError),
    #[error("Invalid credentials: {0}")]
    InvalidCredentials(&'static str),
    #[error("UTF-8 error: {0}")]
    Utf8Error(#[from] FromUtf8Error),
    #[error("Unsupported login mechanism: {0}")]
    UnsupportedLoginMechanism(LoginMechanism),
}

#[derive(PartialEq, Eq, EnumIs, Clone, EnumOfKeys)]
#[enum_of_keys(LoginMechanism)]
#[enum_attr(derive(
    Debug,
    Clone,
    PartialEq,
    Eq,
    EnumIs,
    EnumString,
    Display,
    AsRefStr,
    IntoStaticStr
))]
#[cfg_attr(feature = "zeroize", derive(zeroize::Zeroize, zeroize::ZeroizeOnDrop))]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(tag = "type", content = "value"))]
#[cfg_attr(
    feature = "rkyv",
    derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)
)]
#[cfg_attr(feature = "rkyv", archive(compare(PartialEq), check_bytes))]
#[non_exhaustive]
pub enum Credentials {
    #[enum_attr(strum(serialize = "PLAIN"))]
    Plain { username: String, password: String },
    #[enum_attr(strum(default))]
    #[enum_of_keys(default = mechanism_name)]
    Other {
        mechanism_name: String,
        parameters: String,
    },
}
impl LoginMechanism {
    /// Parses a string into a list of AuthMethods by [RFC 4954](https://datatracker.ietf.org/doc/html/rfc4954#section-3)
    pub fn from_iter<'a>(mechanisms: impl Iterator<Item = &'a str>) -> Vec<LoginMechanism> {
        mechanisms
            .map(|v| LoginMechanism::try_from(v).expect("Unable to parse Auth Method"))
            .collect()
    }
    /// Formats a list of AuthMethods into a string by [RFC 4954](https://datatracker.ietf.org/doc/html/rfc4954#section-3)
    pub fn format_iter<'a>(
        iter: impl Iterator<Item = &'a LoginMechanism> + ExactSizeIterator,
    ) -> String {
        let size = iter.len();
        let mut string = String::with_capacity(size * 5);
        for (index, value) in iter.enumerate() {
            string.push_str(value.as_ref());
            if index != size - 1 {
                string.push(' ');
            }
        }
        string
    }
}
impl Debug for Credentials {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Credentials::Plain { .. } => f.debug_struct("Credentials::Plain").finish(),
            Credentials::Other { mechanism_name, .. } => f
                .debug_struct("Credentials::Other")
                .field("mechanism_name", mechanism_name)
                .finish(),
        }
    }
}
impl TryInto<CredentialValue> for Credentials {
    type Error = CredentialError;
    fn try_into(self) -> Result<CredentialValue, Self::Error> {
        match &self {
            Credentials::Plain { username, password } => {
                let mut user_name_pass = format!("{}:{}", username, password);
                let base64 = STANDARD.encode(user_name_pass.as_bytes());
                #[cfg(feature = "zeroize")]
                zeroize::Zeroize::zeroize(&mut user_name_pass);
                Ok(CredentialValue {
                    login_mechanism: LoginMechanism::Plain,
                    value: base64,
                })
            }
            unsupported => Err(CredentialError::UnsupportedLoginMechanism(
                unsupported.get_key(),
            )),
        }
    }
}
impl TryInto<Credentials> for CredentialValue {
    type Error = CredentialError;

    fn try_into(self) -> Result<Credentials, Self::Error> {
        match &self.login_mechanism {
            LoginMechanism::Plain => {
                let decoded = STANDARD.decode(self.value.as_bytes())?;
                let mut decoded = String::from_utf8(decoded)?;
                let mut split = decoded.split(':');
                let username = split
                    .next()
                    .ok_or(CredentialError::InvalidCredentials("Username not found"))?
                    .to_string();
                let password = split
                    .next()
                    .ok_or(CredentialError::InvalidCredentials("Password not found"))?
                    .to_string();
                drop(split);
                #[cfg(feature = "zeroize")]
                zeroize::Zeroize::zeroize(&mut decoded);
                Ok(Credentials::Plain { username, password })
            }
            v => Err(CredentialError::UnsupportedLoginMechanism(v.clone())),
        }
    }
}
#[derive(PartialEq, Eq)]
#[cfg_attr(feature = "zeroize", derive(zeroize::Zeroize, zeroize::ZeroizeOnDrop))]
pub struct CredentialValue {
    #[cfg_attr(feature = "zeroize", zeroize(skip))]
    pub login_mechanism: LoginMechanism,
    pub value: String,
}

impl Debug for CredentialValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CredentialValue")
            .field("login_mechanism", &self.login_mechanism)
            .finish()
    }
}
impl Display for CredentialValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {}", self.login_mechanism, self.value)
    }
}

#[cfg(test)]
mod tests {
    pub use super::*;

    #[test]
    pub fn test_plain() {
        let credentials = Credentials::Plain {
            username: "username".to_string(),
            password: "password".to_string(),
        };

        let credential_value: CredentialValue = credentials.try_into().unwrap();

        assert_eq!(credential_value.login_mechanism, LoginMechanism::Plain);

        let credentials: Credentials = credential_value.try_into().unwrap();

        assert_eq!(
            credentials,
            Credentials::Plain {
                username: "username".to_string(),
                password: "password".to_string()
            }
        );
    }
}
