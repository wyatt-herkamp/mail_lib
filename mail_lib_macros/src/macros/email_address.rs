use proc_macro2::TokenStream;
use quote::quote;
use syn::{
    parse::{Parse, ParseStream},
    Error, LitInt, LitStr, Result,
};
/// Parse takes in a String Literal then locates the `@` symbol.
#[derive(Debug)]
pub struct EmailAddressMacro {
    email_address: LitStr,
    index: LitInt,
}

impl Parse for EmailAddressMacro {
    fn parse(input: ParseStream) -> Result<Self> {
        let email_address: LitStr = input.parse()?;
        let value = email_address.value();
        let index = value.rfind('@').map(|i| i as usize);
        let index = match index {
            Some(index) => index,
            None => return Err(Error::new(email_address.span(), "Invalid Email Address")),
        };
        let index = LitInt::new(&index.to_string(), email_address.span());
        Ok(EmailAddressMacro {
            email_address,
            index,
        })
    }
}

impl EmailAddressMacro {
    /// The output of this macro is a static EmailAddress
    /// Uses EmailAddress::new_unchecked_raw
    pub fn output(&self) -> TokenStream {
        let Self {
            email_address,
            index,
        } = self;
        quote! {
            unsafe{
                EmailAddress::new_unchecked_raw(#email_address, #index)
            }
        }
    }
}
