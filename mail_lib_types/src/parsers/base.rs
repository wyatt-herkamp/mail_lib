
use chumsky::prelude::*;


pub fn wsp<'a>() -> impl Parser<'a, &'a str, char, super::ErrType<'a>> {
    choice((just(' '), just('\t')))
}
pub fn fws<'a>() -> impl Parser<'a, &'a str, Option<String>, super::ErrType<'a>> {
    wsp()
        .map(|v| v.to_string())
        .or_not()
        .then_ignore(wsp().ignored().repeated())
}
pub fn word<'a>() -> impl Parser<'a, &'a str, String, super::ErrType<'a>> {
    choice((quoted_content(), atom()))
}
pub fn atom<'a>() -> impl Parser<'a, &'a str, String, super::ErrType<'a>> {
    fws()
        .then(atext().repeated().at_least(1).collect::<String>())
        .map(|(wsp, atext)| {
            if let Some(wsp) = wsp {
                format!("{}{}", wsp, atext)
            } else {
                atext
            }
        })
}
pub fn pharse<'a>() -> impl Parser<'a, &'a str, String, super::ErrType<'a>> {
    word()
}
pub fn atext<'a>() -> impl Parser<'a, &'a str, char, super::ErrType<'a>> {
    choice((
        just('!'),
        just('#'),
        just('$'),
        just('%'),
        just('&'),
        just('\''),
        just('*'),
        just('+'),
        just('-'),
        just('/'),
        just('='),
        just('?'),
        just('^'),
        just('_'),
        just('`'),
        just('{'),
        just('|'),
        just('}'),
        just('~'),
        one_of('a'..='z'),
        one_of('A'..='Z'),
        one_of('0'..='9'),
    ))
}
pub fn addr_spec<'a>() -> impl Parser<'a, &'a str, (String, String), super::ErrType<'a>> {
    local_part().then_ignore(just('@')).then(domain())
}

pub fn local_part<'a>() -> impl Parser<'a, &'a str, String, super::ErrType<'a>> {
    choice((dot_atom(), quoted_content(), obs_local_part()))
}

pub fn domain<'a>() -> impl Parser<'a, &'a str, String, super::ErrType<'a>> {
    choice((dot_atom(), obs_domain()))
}

pub fn dot_atom<'a>() -> impl Parser<'a, &'a str, String, super::ErrType<'a>> {
    fws().then(dot_atom_text()).map(|(wsp, atext)| {
        if let Some(wsp) = wsp {
            format!("{}{}", wsp, atext)
        } else {
            atext
        }
    })
}
pub fn dot_atom_text<'a>() -> impl Parser<'a, &'a str, String, super::ErrType<'a>> {
    atext()
        .repeated()
        .at_least(1)
        .collect::<String>()
        .separated_by(just('.'))
        .collect::<Vec<_>>()
        .map(|v| v.join("."))
}

fn text<'a>() -> impl Parser<'a, &'a str, char, super::ErrType<'a>> {
    any().filter(|c| matches!(u32::from(*c), 1..=9 | 11 | 12 | 14..=127))
}
fn quoted_pair<'a>() -> impl Parser<'a, &'a str, char, super::ErrType<'a>> {
    just('\\').ignore_then(text())
}
fn qtext<'a>() -> impl Parser<'a, &'a str, char, super::ErrType<'a>> {
    choice((atext(), just(' ')))
}

pub fn quoted_content<'a>() -> impl Parser<'a, &'a str, String, super::ErrType<'a>> {
    choice((quoted_pair(), qtext()))
        .repeated()
        .collect::<Vec<_>>()
        .delimited_by(just('"'), just('"'))
        .map(|v| {
            v.into_iter()
                .map(|c| c.to_string())
                .collect::<Vec<_>>()
                .join("")
        })
}

pub fn obs_domain<'a>() -> impl Parser<'a, &'a str, String, super::ErrType<'a>> {
    atom()
        .separated_by(just('.'))
        .collect::<Vec<_>>()
        .map(|v| v.join("."))
}
/// Multiple words combined by dots
pub fn obs_local_part<'a>() -> impl Parser<'a, &'a str, String, super::ErrType<'a>> {
    word()
        .separated_by(just('.'))
        .collect::<Vec<_>>()
        .map(|v| v.join("."))
}
#[cfg(test)]
mod tests {
    #[test]
    pub fn test_local() {}
}
