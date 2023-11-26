use mail_lib_types::mail_box::MailBox;
use pretty_assertions::assert_eq;

#[path = "./data/data_types.rs"]
pub mod data_types;

#[test]
pub fn valid_tests() {
    let tests = data_types::build_valid_tests();

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
    let tests = data_types::build_valid_mailboxes();

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
    let tests = data_types::build_invalid_tests();

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
