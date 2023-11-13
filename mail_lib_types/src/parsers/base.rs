use chumsky::prelude::*;
pub fn wsp<'a>() -> impl Parser<'a, &'a str, char, extra::Err<Cheap>> {
    choice((just(' '), just('\t')))
}
pub fn fws<'a>() -> impl Parser<'a, &'a str, Option<String>, extra::Err<Cheap>> {
    wsp()
        .map(|v| v.to_string())
        .or_not()
        .then_ignore(wsp().ignored().repeated())
}
pub fn word<'a>() -> impl Parser<'a, &'a str, String, extra::Err<Cheap>> {
    choice((quoted_content(), atom()))
}
pub fn atom<'a>() -> impl Parser<'a, &'a str, String, extra::Err<Cheap>> {
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
pub fn pharse<'a>() -> impl Parser<'a, &'a str, String, extra::Err<Cheap>> {
    word()
}
pub fn atext<'a>() -> impl Parser<'a, &'a str, char, extra::Err<Cheap>> {
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
pub fn addr_spec<'a>() -> impl Parser<'a, &'a str, (String, String), extra::Err<Cheap>> {
    local_part().then_ignore(just('@')).then(domain())
}

pub fn local_part<'a>() -> impl Parser<'a, &'a str, String, extra::Err<Cheap>> {
    choice((dot_atom(), quoted_content(), obs_local_part()))
}

pub fn domain<'a>() -> impl Parser<'a, &'a str, String, extra::Err<Cheap>> {
    choice((dot_atom(), obs_domain()))
}

pub fn dot_atom<'a>() -> impl Parser<'a, &'a str, String, extra::Err<Cheap>> {
    fws().then(dot_atom_text()).map(|(wsp, atext)| {
        if let Some(wsp) = wsp {
            format!("{}{}", wsp, atext)
        } else {
            atext
        }
    })
}
pub fn dot_atom_text<'a>() -> impl Parser<'a, &'a str, String, extra::Err<Cheap>> {
    atext()
        .repeated()
        .at_least(1)
        .collect::<String>()
        .separated_by(just('.'))
        .collect::<Vec<_>>()
        .map(|v| v.join("."))
}

fn text<'a>() -> impl Parser<'a, &'a str, char, extra::Err<Cheap>> {
    any().filter(|c| matches!(u32::from(*c), 1..=9 | 11 | 12 | 14..=127))
}
fn quoted_pair<'a>() -> impl Parser<'a, &'a str, char, extra::Err<Cheap>> {
    just('\\').ignore_then(text())
}
fn qtext<'a>() -> impl Parser<'a, &'a str, char, extra::Err<Cheap>> {
    choice((atext(), just(' ')))
}
pub fn qcontent<'a>() -> impl Parser<'a, &'a str, String, extra::Err<Cheap>> {
    choice((quoted_pair(), qtext()))
        .repeated()
        .collect::<Vec<_>>()
        .map(|v| v.into_iter().collect::<String>())
}
pub fn quoted_content<'a>() -> impl Parser<'a, &'a str, String, extra::Err<Cheap>> {
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

pub fn obs_domain<'a>() -> impl Parser<'a, &'a str, String, extra::Err<Cheap>> {
    atom()
        .separated_by(just('.'))
        .collect::<Vec<_>>()
        .map(|v| v.join("."))
}
/// Multiple words combined by dots
pub fn obs_local_part<'a>() -> impl Parser<'a, &'a str, String, extra::Err<Cheap>> {
    word()
        .separated_by(just('.'))
        .collect::<Vec<_>>()
        .map(|v| v.join("."))
}
