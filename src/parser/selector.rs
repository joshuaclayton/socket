use super::super::context::Selector;
use nom::{
    branch::alt,
    bytes::complete::{tag, take_while, take_while1},
    character::complete::digit1,
    combinator::{map, map_res, recognize},
    multi::many0,
    sequence::{preceded, terminated},
    IResult,
};

pub fn parse(input: &str) -> IResult<&str, Vec<Selector>> {
    let (input, first) = first_selector(input)?;
    let (input, mut selectors) = many0(subsequent_selector)(input)?;

    selectors.insert(0, first);
    Ok((input, selectors))
}

fn first_selector(input: &str) -> IResult<&str, Selector> {
    alt((
        parse_selector_object_index,
        parse_selector_array_index,
        parse_selector_key,
    ))(input)
}

fn subsequent_selector(input: &str) -> IResult<&str, Selector> {
    alt((
        preceded(tag("."), parse_selector_object_index),
        parse_selector_array_index,
        preceded(tag("."), parse_selector_key),
    ))(input)
}

fn parse_selector_object_index(input: &str) -> IResult<&str, Selector> {
    map(parse_usize, Selector::Index)(input)
}

fn parse_index(input: &str) -> IResult<&str, Selector> {
    alt((
        parse_selector_object_index,
        map(
            take_while(|c: char| c.is_alphanumeric()),
            Selector::KeyedIndex,
        ),
    ))(input)
}

fn parse_selector_array_index(input: &str) -> IResult<&str, Selector> {
    preceded(tag("["), terminated(parse_index, tag("]")))(input)
}

fn parse_selector_key(input: &str) -> IResult<&str, Selector> {
    map(take_while(|c: char| c.is_alphanumeric()), Selector::Key)(input)
}

fn parse_usize(input: &str) -> IResult<&str, usize> {
    map_res(recognize(digit1), str::parse)(input)
}

#[cfg(test)]
mod tests {
    use super::super::super::context::Selector;
    #[test]
    fn selector_parser() {
        assert_eq!(
            super::parse("foo.bar[0].one.1.nested").unwrap(),
            (
                "",
                vec![
                    Selector::Key("foo"),
                    Selector::Key("bar"),
                    Selector::Index(0),
                    Selector::Key("one"),
                    Selector::Index(1),
                    Selector::Key("nested"),
                ]
            )
        )
    }

    #[test]
    fn parse_text_key() {
        assert_eq!(
            super::parse("bar[baz]").unwrap(),
            ("", vec![Selector::Key("bar"), Selector::KeyedIndex("baz"),])
        )
    }
}
