use super::{custom_attributes, Attribute, Tag};
use nom::{
    branch::alt,
    bytes::complete::{tag, take_while},
    combinator::{map, opt},
    multi::many1,
    sequence::preceded,
    IResult,
};

pub fn parse(input: &str) -> IResult<&str, Tag> {
    alt((parse_explicit_tag, parse_implicit_tag))(input)
}

fn parse_implicit_tag(input: &str) -> IResult<&str, Tag> {
    let (input, mut attributes) = parse_attributes(input)?;

    let (input, custom_attributes) = opt(custom_attributes::parse)(input)?;

    if let Some(customs) = custom_attributes {
        attributes.extend(customs);
    }

    Ok((
        input,
        Tag {
            name: "div",
            attributes,
        },
    ))
}

fn parse_explicit_tag(input: &str) -> IResult<&str, Tag> {
    let (input, name) = preceded(tag("%"), take_while(|c: char| c.is_alphanumeric()))(input)?;
    let (input, mut attributes) = map(opt(parse_attributes), |v| v.unwrap_or(vec![]))(input)?;
    let (input, custom_attributes) = opt(custom_attributes::parse)(input)?;

    if let Some(customs) = custom_attributes {
        attributes.extend(customs);
    }
    Ok((input, Tag { name, attributes }))
}

fn parse_attributes(input: &str) -> IResult<&str, Vec<Attribute>> {
    let parse_class = map(preceded(tag("."), parse_html_class), Attribute::Class);
    let parse_id = map(preceded(tag("#"), parse_html_class), Attribute::Id);
    let (input, attributes) = many1(alt((parse_class, parse_id)))(input)?;
    Ok((input, attributes))
}

fn parse_html_class(input: &str) -> IResult<&str, &str> {
    take_while(|c: char| c.is_alphanumeric() || c == '-' || c == '_' || c == '/' || c == ':' || c == '[' || c == ']')(input)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tailwind_class() {
        assert_eq!(parse_html_class("w-3/4").unwrap(), ("", "w-3/4"));
    }

    #[test]
    fn tailwind_bracket_class() {
        assert_eq!(parse_html_class("h-[66vh]").unwrap(), ("", "h-[66vh]"));
    }
}
