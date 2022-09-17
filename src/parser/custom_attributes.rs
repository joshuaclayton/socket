use super::super::{Attribute, AttributeValueComponent};
use super::selector;
use nom::{
    branch::alt,
    bytes::complete::{tag, take_till1, take_while},
    combinator::map,
    multi::{many0, separated_list1},
    sequence::{pair, preceded, terminated},
    IResult,
};

fn wrapped_string(input: &str) -> IResult<&str, Vec<AttributeValueComponent>> {
    let (input, base) = preceded(tag("\""), alt((raw_quoted, interpolated)))(input)?;
    let (input, mut rest) = terminated(many0(alt((raw_quoted, interpolated))), tag("\""))(input)?;

    rest.insert(0, base);

    Ok((input, rest))
}

fn raw_quoted(input: &str) -> IResult<&str, AttributeValueComponent> {
    map(
        take_till1(|c: char| c == '\"' || c == '{'),
        AttributeValueComponent::RawValue,
    )(input)
}

fn raw_unwrapped(input: &str) -> IResult<&str, AttributeValueComponent> {
    map(
        take_till1(|c: char| c.is_whitespace() || c == '=' || c == ')' || c == '{'),
        AttributeValueComponent::RawValue,
    )(input)
}

fn interpolated(input: &str) -> IResult<&str, AttributeValueComponent> {
    let parse_interpolated_selectors = preceded(tag("{"), terminated(selector::parse, tag("}")));
    map(
        parse_interpolated_selectors,
        AttributeValueComponent::InterpolatedValue,
    )(input)
}

fn unwrapped_string(input: &str) -> IResult<&str, Vec<AttributeValueComponent>> {
    let (input, base) = alt((interpolated, raw_unwrapped))(input)?;
    let (input, mut rest) = many0(alt((interpolated, raw_unwrapped)))(input)?;

    rest.insert(0, base);

    Ok((input, rest))
}

pub fn parse(input: &str) -> IResult<&str, Vec<Attribute>> {
    let attribute_name = take_while(|c: char| c.is_alphanumeric() || c == '-' || c == '_');
    let parse_pair = pair(
        terminated(attribute_name, tag("=")),
        alt((wrapped_string, unwrapped_string)),
    );
    let parse_attribute = map(parse_pair, |(k, v)| Attribute::Custom(k, v));

    preceded(
        tag("("),
        terminated(separated_list1(tag(" "), parse_attribute), tag(")")),
    )(input)
}

#[cfg(test)]
mod tests {
    use super::super::super::{context::Selector, Attribute, AttributeValueComponent};

    #[test]
    fn just_custom_attributes() {
        fn raw_custom_attribute<'a>(value: &'a str) -> Vec<AttributeValueComponent<'a>> {
            vec![AttributeValueComponent::RawValue(value)]
        }

        assert_eq!(
            super::parse("(lang=en)"),
            Ok((
                "",
                vec![Attribute::Custom("lang", raw_custom_attribute("en"))]
            ))
        );

        assert_eq!(
            super::parse("(http-equiv=x-ua-compatible content=\"ie=edge\")").unwrap(),
            (
                "",
                vec![
                    Attribute::Custom("http-equiv", raw_custom_attribute("x-ua-compatible")),
                    Attribute::Custom("content", raw_custom_attribute("ie=edge"))
                ]
            )
        );
    }

    #[test]
    fn single_unwrapped_string() {
        assert_eq!(
            ("", vec![AttributeValueComponent::RawValue("foo")]),
            super::unwrapped_string("foo").unwrap()
        );

        assert_eq!(
            (
                "",
                vec![AttributeValueComponent::InterpolatedValue(vec![
                    Selector::Key("foo"),
                    Selector::Key("bar")
                ])]
            ),
            super::unwrapped_string("{foo.bar}").unwrap()
        );

        assert_eq!(
            (
                "",
                vec![
                    AttributeValueComponent::RawValue("starting"),
                    AttributeValueComponent::InterpolatedValue(vec![
                        Selector::Key("foo"),
                        Selector::Key("bar")
                    ])
                ]
            ),
            super::unwrapped_string("starting{foo.bar}").unwrap()
        );
    }

    #[test]
    fn single_wrapped_string() {
        assert_eq!(
            Ok(("", vec![AttributeValueComponent::RawValue("foo")])),
            super::wrapped_string("\"foo\"")
        );
    }

    #[test]
    fn single_wrapped_interpolation() {
        assert_eq!(
            (
                "",
                vec![AttributeValueComponent::InterpolatedValue(vec![
                    Selector::Key("foo"),
                    Selector::Key("bar")
                ])]
            ),
            super::wrapped_string("\"{foo.bar}\"").unwrap()
        );

        assert_eq!(
            (
                "",
                vec![
                    AttributeValueComponent::RawValue("starting"),
                    AttributeValueComponent::InterpolatedValue(vec![
                        Selector::Key("foo"),
                        Selector::Key("bar")
                    ])
                ]
            ),
            super::wrapped_string("\"starting{foo.bar}\"").unwrap()
        );
    }
}
