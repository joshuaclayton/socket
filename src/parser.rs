mod custom_attributes;
mod selector;
use super::{Attribute, Node, Nodes, Tag};
use nom::{
    branch::alt,
    bytes::complete::{tag, take_till, take_while},
    combinator::{map, opt},
    multi::{count, many0, many1, separated_list},
    sequence::{preceded, terminated},
    IResult,
};
use std::path::PathBuf;

pub fn parse(input: &str) -> IResult<&str, Nodes> {
    let (input, html_attributes) = opt(terminated(
        preceded(tag("!HTML"), opt(custom_attributes::parse)),
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
        let (input, selectors) = terminated(selector::parse, tag("\n"))(input)?;
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

fn parse_fragment(input: &str) -> IResult<&str, Node> {
    let (input, path) = map(preceded(tag("- fragment "), to_newline), PathBuf::from)(input)?;

    Ok((input, Node::Fragment { path }))
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
        let (input, contents) = map(selector::parse, Node::InterpolatedText)(input)?;
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
    let (input, custom_attributes) = opt(custom_attributes::parse)(input)?;

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

fn parse_tag(input: &str) -> IResult<&str, Tag> {
    alt((parse_explicit_tag, parse_implicit_tag))(input)
}

fn parse_node(depth: usize) -> Box<dyn Fn(&str) -> IResult<&str, Node>> {
    Box::new(move |input| {
        let (input, _) = count(tag("  "), depth)(input)?;
        alt((
            parse_for_loop(depth),
            parse_fragment,
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
    fn unquoted_custom_attribute() {
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
    fn attribute_interpolation() {
        assert_eq!(
            Socket::parse("%a(href=mailto:{contact.email})= contact.name")
                .unwrap()
                .with_context("{\"contact\": {\"email\": \"person@example.com\", \"name\": \"Person's name\"}}")
                .to_html(),
            "<a href=\"mailto:person@example.com\">Person's name</a>"
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
    fn fragments() {
        use std::collections::HashMap;
        use std::path::PathBuf;

        let mut fragments: HashMap<PathBuf, String> = HashMap::new();
        fragments.insert(PathBuf::from("foo/item.skt"), "%li= item.name".into());

        assert_eq!(
            Socket::parse(
                "%ul\n  - for item in items\n    - fragment foo/item.skt\n    %li Separator"
            )
            .unwrap()
            .with_fragments(&fragments)
            .with_context("{\"items\": [{\"name\": \"Jane\"}, {\"name\": \"John\"}]}")
            .to_html(),
            "<ul><li>Jane</li><li>Separator</li><li>John</li><li>Separator</li></ul>",
        )
    }
}
