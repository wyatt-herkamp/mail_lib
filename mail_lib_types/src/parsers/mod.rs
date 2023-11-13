pub mod base;
pub mod mail_box;

pub fn is_atom_text(v: &str) -> bool {
    for ele in v.chars() {
        if ele.is_ascii_alphanumeric() {
            continue;
        }

        match ele {
            '!' | '#' | '$' | '%' | '&' | '\'' | '*' | '+' | '-' | '/' | '=' | '?' | '^' | '_'
            | '`' | '{' | '|' | '}' | '~' => continue,
            _ => return false,
        }
    }
    true
}
