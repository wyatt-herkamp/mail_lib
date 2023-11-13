/*!
Tests for Email Address Validation
Feel free to add more tests to the `data/valid_emails.json` and `data/invalid_emails.json` files.
 */
use mail_lib_types::email_address::{EmailErrorMessage, EmailPart};
use pretty_assertions::assert_eq;
use serde::{Deserialize, Serialize};

static VALID_EMAILS: &str = include_str!("./data/valid_emails.json");
static INVALID_EMAILS: &str = include_str!("./data/invalid_emails.json");
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
        let email_address = mail_lib_types::email_address::EmailAddress::new(&test.email);
        let unsafe_test = unsafe {
            mail_lib_types::email_address::EmailAddress::new_only_check_for_at(&test.email)
        };
        assert_eq!(email_address, unsafe_test);
        match email_address {
            Ok(email_address) => {
                assert_eq!(email_address.get_local(), test.local);
                assert_eq!(email_address.get_domain(), test.domain);
            }
            Err(e) => {
                if let EmailErrorMessage::InvalidCharacter(c, pos) = e.error_message {
                    println!("{}", test.email);
                    println!("{}^", " ".repeat(pos));
                    panic!("Invalid character `{}` at position {}", c, pos)
                }
                panic!("{:?}: {}", test.email, e);
            }
        }
    }
}

#[test]
pub fn invalid_tests() {
    let tests = build_invalid_tests();

    for test in tests {
        println!("{:?}", test);
        let email_address = mail_lib_types::email_address::EmailAddress::new(&test.email);

        match email_address {
            Ok(email_address) => {
                eprintln!("Email Address {:?} is Valid", test.email);
                panic!("Email Address {:?} is Valid", email_address)
            }
            Err(e) => {
                assert_eq!(e.error_part, test.part)
            }
        }
    }
}
