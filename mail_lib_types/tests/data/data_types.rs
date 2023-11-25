/*!
Tests for Email Address Validation
Feel free to add more tests to the `data/valid_emails.json` and `data/invalid_emails.json` files.
 */
use mail_lib_types::email_address::EmailPart;
use serde::{Deserialize, Serialize};
static VALID_EMAILS: &str = include_str!("./valid_emails.json");
static VALID_MAILBOXES: &str = include_str!("./valid_mailboxes.json");
static INVALID_EMAILS: &str = include_str!("./invalid_emails.json");

#[derive(Serialize, Deserialize, Debug)]
pub struct ValidMailbox {
    pub mailbox: String,
    pub name: String,
    pub local: String,
    pub domain: String,
}
#[derive(Serialize, Deserialize, Debug)]
pub struct ValidEmail {
    pub email: String,
    pub local: String,
    pub domain: String,
}
#[derive(Serialize, Deserialize, Debug)]
pub struct InvalidEmail {
    pub email: String,
    pub part: Option<EmailPart>,
}

pub fn build_valid_mailboxes() -> Vec<ValidMailbox> {
    serde_json::from_str(VALID_MAILBOXES).expect("Unable to Parse the Valid Mailboxes")
}

pub fn build_valid_tests() -> Vec<ValidEmail> {
    serde_json::from_str(VALID_EMAILS).expect("Unable to Parse the Valid Emails")
}

pub fn build_invalid_tests() -> Vec<InvalidEmail> {
    serde_json::from_str(INVALID_EMAILS).expect("Unable to Parse the Invalid Emails")
}
