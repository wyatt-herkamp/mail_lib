
pub mod base;
pub mod mail_box;
pub mod rfcs;

#[cfg(not(feature = "ariadne"))]
pub(crate) type ErrType<'a> = chumsky::extra::Err<chumsky::error::Cheap>;
#[cfg(feature = "ariadne")]
pub(crate) type ErrType<'a> = chumsky::extra::Err<chumsky::error::Rich<'a, char>>;
