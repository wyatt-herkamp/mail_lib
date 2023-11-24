use mail_lib_types::{
    email_address::{EmailErrorMessage, EmailPart},
    mail_box::MailBox,
};
use pretty_assertions::assert_eq;
use serde::{Deserialize, Serialize};
static VALID_EMAILS: &str = include_str!("./data/valid_emails.json");
static VALID_MAILBOXES: &str = include_str!("./data/valid_mailboxes.json");
static INVALID_EMAILS: &str = include_str!("./data/invalid_emails.json");
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
fn build_valid_mailboxes() -> Vec<ValidMailbox> {
    serde_json::from_str(VALID_MAILBOXES).expect("Unable to Parse the Valid Mailboxes")
}

fn build_valid_tests() -> Vec<ValidEmail> {
    serde_json::from_str(VALID_EMAILS).expect("Unable to Parse the Valid Emails")
}

fn build_invalid_tests() -> Vec<InvalidEmail> {
    serde_json::from_str(INVALID_EMAILS).expect("Unable to Parse the Invalid Emails")
}

#[test]
pub fn valid_tests() {
    let tests = build_valid_tests();

    for test in tests {
        println!("Testing Email {:?}", test);
        let email_address = MailBox::try_from(test.email.as_str());
        match email_address {
            Ok(email_address) => {
                assert_eq!(email_address.get_local(), test.local);
                assert_eq!(email_address.get_domain(), test.domain);
            }
            Err(e) => {
                panic!("{:?}: {}", test.email, e)
            }
        }
    }
}
#[test]
pub fn valid_mailbox_test() {
    let tests = build_valid_mailboxes();

    for test in tests {
        println!("Testing Email {:?}", test);
        let email_address = MailBox::try_from(test.mailbox.as_str());
        match email_address {
            Ok(email_address) => {
                assert_eq!(email_address.get_local(), test.local);
                assert_eq!(email_address.get_domain(), test.domain);
                
            }
            Err(e) => {
                panic!("{:?}: {}", test.mailbox, e)
            }
        }
    }
}
#[test]
pub fn invalid_tests() {
    let tests = build_invalid_tests();

    for test in tests {
        println!("{:?}", test);
        let email_address = MailBox::try_from(test.email.as_str());

        match email_address {
            Ok(email_address) => {
                eprintln!("Email Address {:?} is Valid", test.email);
                panic!("Email Address {:?} is Valid", email_address)
            }
            Err(e) => {
                println!("{}", e);
            }
        }
    }
}
