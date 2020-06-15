use super::{context::Selector, Attribute, Node, Nodes, Tag};
use nom::{
    branch::alt,
    bytes::complete::{tag, take_till, take_while},
    character::complete::digit1,
    combinator::{map, map_res, opt, recognize},
    multi::{count, many0, many1, separated_list},
    sequence::{pair, preceded, terminated},
    IResult,
};

pub fn parse(input: &str) -> IResult<&str, Nodes> {
    let (input, html_attributes) = opt(terminated(
        preceded(tag("!HTML"), opt(parse_custom_attributes)),
        tag("\n"),
    ))(input)?;
    let (input, children) = parse_nodes(0)(input)?;

    match html_attributes {
        None => Ok((input, children)),
        Some(attributes) => {
            let tag = Tag {
                name: "html",
                attributes: attributes.unwrap_or(vec![]),
            };
            let root = Node::Element { tag, children };
            Ok((input, Nodes::new_document(vec![root])))
        }
    }
}

fn parse_nodes(depth: usize) -> Box<dyn Fn(&str) -> IResult<&str, Nodes>> {
    Box::new(move |input| {
        map(
            separated_list(tag("\n"), preceded(many0(tag("\n")), parse_node(depth))),
            Nodes::new_fragment,
        )(input)
    })
}

fn parse_text_node(input: &str) -> IResult<&str, Node> {
    map(to_newline, Node::Text)(input)
}

fn parse_for_loop(depth: usize) -> Box<dyn Fn(&str) -> IResult<&str, Node>> {
    Box::new(move |input| {
        let (input, local) = preceded(
            tag("- for "),
            terminated(take_while(|c: char| c.is_alphanumeric()), tag(" in ")),
        )(input)?;
        let (input, selectors) = terminated(parse_selector, tag("\n"))(input)?;
        let (input, children) = parse_nodes(depth + 1)(input)?;

        Ok((
            input,
            Node::ForLoop {
                local,
                selectors,
                children,
            },
        ))
    })
}

fn parse_node_with_text(depth: usize) -> Box<dyn Fn(&str) -> IResult<&str, Node>> {
    Box::new(move |input| {
        let (input, tag) = terminated(parse_tag, tag(" "))(input)?;
        let (input, contents) = map(to_newline, Node::Text)(input)?;
        let (input, mut children) = parse_nodes(depth + 1)(input)?;
        children.prepend(contents);

        Ok((input, Node::Element { tag, children }))
    })
}

fn parse_node_with_interpolated_text(depth: usize) -> Box<dyn Fn(&str) -> IResult<&str, Node>> {
    Box::new(move |input| {
        let (input, tag) = terminated(parse_tag, tag("= "))(input)?;
        let (input, contents) = map(parse_selector, Node::InterpolatedText)(input)?;
        let (input, mut children) = parse_nodes(depth + 1)(input)?;
        children.prepend(contents);

        Ok((input, Node::Element { tag, children }))
    })
}

fn parse_node_without_text(depth: usize) -> Box<dyn Fn(&str) -> IResult<&str, Node>> {
    Box::new(move |input| {
        let (input, tag) = parse_tag(input)?;
        let (input, children) = parse_nodes(depth + 1)(input)?;

        Ok((input, Node::Element { tag, children }))
    })
}

fn parse_explicit_tag(input: &str) -> IResult<&str, Tag> {
    let (input, name) = preceded(tag("%"), take_while(|c: char| c.is_alphanumeric()))(input)?;
    let (input, mut attributes) = map(opt(parse_attributes), |v| v.unwrap_or(vec![]))(input)?;
    let (input, custom_attributes) = opt(parse_custom_attributes)(input)?;

    if let Some(customs) = custom_attributes {
        attributes.extend(customs);
    }
    Ok((input, Tag { name, attributes }))
}

fn parse_html_class(input: &str) -> IResult<&str, &str> {
    take_while(|c: char| c.is_alphanumeric() || c == '-' || c == '_')(input)
}

fn parse_attributes(input: &str) -> IResult<&str, Vec<Attribute>> {
    let parse_class = map(preceded(tag("."), parse_html_class), Attribute::Class);
    let parse_id = map(preceded(tag("#"), parse_html_class), Attribute::Id);
    let (input, attributes) = many1(alt((parse_class, parse_id)))(input)?;
    Ok((input, attributes))
}

fn parse_custom_attributes(input: &str) -> IResult<&str, Vec<Attribute>> {
    let attribute_name = take_while(|c: char| c.is_alphanumeric() || c == '-' || c == '_');
    let wrapped_string = terminated(preceded(tag("\""), take_till(|c| c == '\"')), tag("\""));
    let unwrapped_string = take_till(|c: char| c.is_whitespace() || c == '=' || c == ')');
    let parse_pair = pair(
        terminated(attribute_name, tag("=")),
        alt((wrapped_string, unwrapped_string)),
    );
    let parse_attribute = map(parse_pair, |(k, v)| Attribute::Custom(k, v));

    preceded(
        tag("("),
        terminated(separated_list(tag(" "), parse_attribute), tag(")")),
    )(input)
}

fn parse_usize(input: &str) -> IResult<&str, usize> {
    map_res(recognize(digit1), str::parse)(input)
}

fn parse_selector_object_index(input: &str) -> IResult<&str, Selector> {
    map(parse_usize, Selector::Index)(input)
}

fn parse_selector_array_index(input: &str) -> IResult<&str, Selector> {
    preceded(tag("["), terminated(parse_selector_object_index, tag("]")))(input)
}

fn parse_selector_key(input: &str) -> IResult<&str, Selector> {
    map(take_while(|c: char| c.is_alphanumeric()), Selector::Key)(input)
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

fn parse_selector(input: &str) -> IResult<&str, Vec<Selector>> {
    let (input, first) = first_selector(input)?;
    let (input, mut selectors) = many0(subsequent_selector)(input)?;

    selectors.insert(0, first);
    Ok((input, selectors))
}

fn parse_implicit_tag(input: &str) -> IResult<&str, Tag> {
    let (input, mut attributes) = parse_attributes(input)?;

    let (input, custom_attributes) = opt(parse_custom_attributes)(input)?;

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

fn parse_tag(input: &str) -> IResult<&str, Tag> {
    alt((parse_explicit_tag, parse_implicit_tag))(input)
}

fn parse_node(depth: usize) -> Box<dyn Fn(&str) -> IResult<&str, Node>> {
    Box::new(move |input| {
        let (input, _) = count(tag("  "), depth)(input)?;
        alt((
            parse_for_loop(depth),
            parse_node_with_text(depth),
            parse_node_with_interpolated_text(depth),
            parse_node_without_text(depth),
            parse_text_node,
        ))(input)
    })
}

pub fn to_newline(input: &str) -> IResult<&str, &str> {
    take_till(|c| c == '\n')(input)
}

#[cfg(test)]
mod tests {
    use super::super::context::Selector;
    use super::super::Attribute;
    use super::super::Socket;

    #[test]
    fn simple_tag() {
        assert_eq!(
            Socket::parse("%h1 Hello world").unwrap().to_html(),
            "<h1>Hello world</h1>"
        );
    }

    #[test]
    fn multiple_lines() {
        assert_eq!(
            Socket::parse("%h1 Hello world\n%h2 Subtitle")
                .unwrap()
                .to_html(),
            "<h1>Hello world</h1><h2>Subtitle</h2>"
        );
    }

    #[test]
    fn multiple_lines_with_additional_newlines() {
        assert_eq!(
            Socket::parse("\n\n%h1 Hello world\n\n%h2 Subtitle\n\n\n\n")
                .unwrap()
                .to_html(),
            "<h1>Hello world</h1><h2>Subtitle</h2>"
        );
    }

    #[test]
    fn nested_elements() {
        assert_eq!(
            Socket::parse("%div Hello world\n  %h2 Subtitle")
                .unwrap()
                .to_html(),
            "<div>Hello world<h2>Subtitle</h2></div>"
        );
    }

    #[test]
    fn multiple_nested_elements() {
        assert_eq!(
            Socket::parse("%div Hello world\n  %h2 Subtitle\n  %h3 Another\n%div\n  %h2 Another subtitle")
                .unwrap()
                .to_html(),
            "<div>Hello world<h2>Subtitle</h2><h3>Another</h3></div><div><h2>Another subtitle</h2></div>"
        );
    }

    #[test]
    fn adjacent_text() {
        assert_eq!(
            Socket::parse("%div\n  text\n  %h2 other\n  text")
                .unwrap()
                .to_html(),
            "<div>text<h2>other</h2>text</div>"
        );
    }

    #[test]
    fn div_with_class() {
        assert_eq!(
            Socket::parse(".custom-class").unwrap().to_html(),
            "<div class=\"custom-class\"></div>"
        );
    }

    #[test]
    fn div_with_classes() {
        assert_eq!(
            Socket::parse(".custom-class.other").unwrap().to_html(),
            "<div class=\"custom-class other\"></div>"
        );
    }

    #[test]
    fn div_with_id() {
        assert_eq!(
            Socket::parse("#unique-id").unwrap().to_html(),
            "<div id=\"unique-id\"></div>"
        );
    }

    #[test]
    fn explicit_tag_with_classes() {
        assert_eq!(
            Socket::parse("%section.custom-class.other")
                .unwrap()
                .to_html(),
            "<section class=\"custom-class other\"></section>"
        );
    }

    #[test]
    fn nested_structure_with_classes_and_ids() {
        assert_eq!(
            Socket::parse("#adjacent\n%section#section-id.other some text\n\n%header#primary\n  %h1.inner  welcome!")
            .unwrap()
            .to_html(),
            "<div id=\"adjacent\"></div><section id=\"section-id\" class=\"other\">some text</section><header id=\"primary\"><h1 class=\"inner\"> welcome!</h1></header>"
        );
    }

    #[test]
    fn custom_attributes() {
        assert_eq!(
            Socket::parse("%img(src=\"example.png\" alt=\"What do you think?\")")
                .unwrap()
                .to_html(),
            "<img src=\"example.png\" alt=\"What do you think?\"></img>"
        );
    }

    #[test]
    fn unquoted_custom_attribut() {
        assert_eq!(
            Socket::parse("%img(src=example.png alt=\"What do you think?\")")
                .unwrap()
                .to_html(),
            "<img src=\"example.png\" alt=\"What do you think?\"></img>"
        );
    }

    #[test]
    fn html_doctype_with_attributes() {
        assert_eq!(
            Socket::parse("!HTML(lang=en)\n%head\n%body\n  %h1 Hello world")
                .unwrap()
                .to_html(),
            "<!DOCTYPE html><html lang=\"en\"><head></head><body><h1>Hello world</h1></body></html>"
        );
    }

    #[test]
    fn html_doctype() {
        assert_eq!(
            Socket::parse("!HTML\n%head\n%body\n  %h1 Hello world")
                .unwrap()
                .to_html(),
            "<!DOCTYPE html><html><head></head><body><h1>Hello world</h1></body></html>"
        );
    }

    #[test]
    fn just_custom_attributes() {
        assert_eq!(
            super::parse_custom_attributes("(lang=en)").unwrap(),
            ("", vec![Attribute::Custom("lang", "en")])
        );

        assert_eq!(
            super::parse_custom_attributes("(http-equiv=x-ua-compatible content=\"ie=edge\")")
                .unwrap(),
            (
                "",
                vec![
                    Attribute::Custom("http-equiv", "x-ua-compatible"),
                    Attribute::Custom("content", "ie=edge")
                ]
            )
        );
    }

    #[test]
    fn simple_interpolation() {
        assert_eq!(
            Socket::parse("%h1= title")
                .unwrap()
                .with_context("{\"title\": \"Hello world\"}")
                .to_html(),
            "<h1>Hello world</h1>"
        );
    }

    #[test]
    fn object_interpolation() {
        assert_eq!(
            Socket::parse("%h1= title.primary\n%h2= title.secondary")
                .unwrap()
                .with_context("{\"title\": {\"primary\": \"Hello world\", \"secondary\": \"wow this works\"}}")
                .to_html(),
            "<h1>Hello world</h1><h2>wow this works</h2>"
        );
    }

    #[test]
    fn for_loops() {
        assert_eq!(
            Socket::parse("%ul\n  - for value in values\n    %li= value")
                .unwrap()
                .with_context("{\"values\": [\"first\", \"second\", \"third\"]}")
                .to_html(),
            "<ul><li>first</li><li>second</li><li>third</li></ul>"
        );
    }

    #[test]
    fn for_loops_with_object() {
        assert_eq!(
            Socket::parse("%ul\n  - for value in values\n    %li= value.name")
                .unwrap()
                .with_context("{\"values\": [{\"name\": \"Jane\"}, {\"name\": \"John\"}]}")
                .to_html(),
            "<ul><li>Jane</li><li>John</li></ul>"
        );
    }

    #[test]
    fn selector_parser() {
        assert_eq!(
            super::parse_selector("foo.bar[0].one.1.nested").unwrap(),
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
}
