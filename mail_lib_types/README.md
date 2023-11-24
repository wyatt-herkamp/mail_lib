# Mail_Lib_Types

Representation of Email Types in Rust. Think of [http](https://github.com/hyperium/http) for Email

## Supported Types

- [X] Email Address - Will Parse and Validate
- [ ] MailBox: Will Parse, Validate, and format.
- [X] Authentication Credientials: [Plain](https://datatracker.ietf.org/doc/html/rfc4616)

## Cargo Features

- `rkyv` - Implements [rkyv::Serialize](https://docs.rs/rkyv/latest/rkyv/trait.Serialize.html) and [rkyv::Deserialize](https://docs.rs/rkyv/latest/rkyv/trait.Deserialize.html) for types. This feature mainly exists for the usage inside nitro_mail
- `serde` - Implements [serde::Serialize](https://docs.rs/serde/latest/serde/trait.Serialize.html) and [serde::Deserialize](https://docs.rs/serde/latest/serde/trait.Deserialize.html)
- `zeroize` - Types that contain Login Details will not implement [Zeroize](https://docs.rs/zeroize/latest/zeroize/)
