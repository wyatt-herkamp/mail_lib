use crate::email_address::InvalidEmailAddress;

pub fn validate_domain(domain: &str) -> Result<(), InvalidEmailAddress> {
    if domain.is_empty() || domain.len() > 255 || !domain.contains(".") {
        Err(InvalidEmailAddress("Invalid Domain"))
    } else {
        Ok(())
    }
}
pub fn validate_user(user: &str) -> Result<(), InvalidEmailAddress> {
    if user.is_empty() || user.len() > 64 {
        Err(InvalidEmailAddress("Invalid user"))
    } else {
        Ok(())
    }
}
