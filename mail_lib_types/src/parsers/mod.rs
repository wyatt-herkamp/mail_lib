#![allow(missing_docs)]
#[doc(hidden)]
pub mod rfcs;

pub type ErrType<'a> = chumsky::extra::Err<chumsky::error::Cheap>;
