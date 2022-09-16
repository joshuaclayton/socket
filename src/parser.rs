mod custom_attributes;
mod selector;
mod tag;

use super::{Attribute, Node, Nodes, Tag};
use nom::{
    branch::alt,
    bytes::complete::{tag, take_till, take_while},
    combinator::{map, opt},
    multi::{count, many0, many1, separated_list0, separated_list1},
    sequence::{preceded, separated_pair, terminated},
    IResult,
};
use std::path::PathBuf;

pub fn parse(input: &str) -> IResult<&str, Nodes> {
    let (input, html_attributes) = opt(terminated(
        preceded(tag("!HTML"), opt(custom_attributes::parse)),
        tag("\n"),
    ))(input)?;
    let (input, children) = alt((parse_fragment_subclass, parse_nodes(0)))(input)?;

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
            separated_list0(tag("\n"), preceded(many0(tag("\n")), parse_node(depth))),
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
                index: None,
                local,
                selectors,
                children,
            },
        ))
    })
}

fn parse_for_loop_with_index(depth: usize) -> Box<dyn Fn(&str) -> IResult<&str, Node>> {
    Box::new(move |input| {
        let (input, (index, local)) = preceded(
            tag("- for "),
            terminated(
                separated_pair(
                    take_while(|c: char| c.is_alphanumeric()),
                    tag(", "),
                    take_while(|c: char| c.is_alphanumeric()),
                ),
                tag(" in "),
            ),
        )(input)?;
        let (input, selectors) = terminated(selector::parse, tag("\n"))(input)?;
        let (input, children) = parse_nodes(depth + 1)(input)?;

        Ok((
            input,
            Node::ForLoop {
                index: Some(index),
                local,
                selectors,
                children,
            },
        ))
    })
}

fn parse_if(depth: usize) -> Box<dyn Fn(&str) -> IResult<&str, Node>> {
    Box::new(move |input| {
        let (input, selectors) =
            preceded(tag("- if "), terminated(selector::parse, tag("\n")))(input)?;
        let (input, true_children) = parse_nodes(depth + 1)(input)?;

        Ok((
            input,
            Node::IfElse {
                selectors,
                true_children,
                false_children: Nodes::default(),
            },
        ))
    })
}

fn parse_if_else(depth: usize) -> Box<dyn Fn(&str) -> IResult<&str, Node>> {
    Box::new(move |input| {
        let (input, selectors) =
            preceded(tag("- if "), terminated(selector::parse, tag("\n")))(input)?;
        let (input, true_children) = parse_nodes(depth + 1)(input)?;
        let (input, _) = preceded(many1(tag("\n")), count(tag("  "), depth))(input)?;
        let (input, _) = terminated(tag("- else"), tag("\n"))(input)?;
        let (input, false_children) = parse_nodes(depth + 1)(input)?;

        Ok((
            input,
            Node::IfElse {
                selectors,
                true_children,
                false_children,
            },
        ))
    })
}

fn parse_markdown_line(depth: usize) -> Box<dyn Fn(&str) -> IResult<&str, &str>> {
    Box::new(move |input| preceded(count(tag("  "), depth), to_newline)(input))
}

fn parse_markdown(depth: usize) -> Box<dyn Fn(&str) -> IResult<&str, Node>> {
    Box::new(move |input| {
        let (input, _) = terminated(tag(":markdown"), many1(tag("\n")))(input)?;
        let (input, markdown) = alt((
            separated_list1(many1(tag("\n")), parse_markdown_line(depth + 1)),
            map(parse_markdown_line(depth + 1), |v| vec![v]),
        ))(input)?;

        Ok((input, Node::Markdown(markdown)))
    })
}
fn parse_fragment(input: &str) -> IResult<&str, Node> {
    let (input, path) = map(preceded(tag("- fragment "), to_newline), PathBuf::from)(input)?;

    Ok((input, Node::Fragment { path }))
}

fn parse_extends(input: &str) -> IResult<&str, PathBuf> {
    map(preceded(tag("- extends "), to_newline), PathBuf::from)(input)
}

fn parse_block_contents(depth: usize) -> Box<dyn Fn(&str) -> IResult<&str, Node>> {
    Box::new(move |input| {
        let (input, name) = preceded(tag("- block "), to_newline)(input)?;
        let (input, children) = parse_nodes(depth + 1)(input)?;

        Ok((input, Node::Block { name, children }))
    })
}

fn parse_node_with_text(depth: usize) -> Box<dyn Fn(&str) -> IResult<&str, Node>> {
    Box::new(move |input| {
        let (input, tag) = terminated(tag::parse, tag(" "))(input)?;
        let (input, contents) = map(to_newline, Node::Text)(input)?;
        let (input, mut children) = parse_nodes(depth + 1)(input)?;
        children.prepend(contents);

        Ok((input, Node::Element { tag, children }))
    })
}

fn parse_interpolated_text(input: &str) -> IResult<&str, Node> {
    map(selector::parse, Node::InterpolatedText)(input)
}

fn parse_block_value(input: &str) -> IResult<&str, Node> {
    map(preceded(tag("block "), to_newline), Node::BlockValue)(input)
}

fn parse_node_with_interpolated_text(depth: usize) -> Box<dyn Fn(&str) -> IResult<&str, Node>> {
    Box::new(move |input| {
        let (input, tag) = terminated(tag::parse, tag("= "))(input)?;
        let (input, contents) = alt((parse_block_value, parse_interpolated_text))(input)?;
        let (input, mut children) = parse_nodes(depth + 1)(input)?;
        children.prepend(contents);

        Ok((input, Node::Element { tag, children }))
    })
}

fn parse_node_without_text(depth: usize) -> Box<dyn Fn(&str) -> IResult<&str, Node>> {
    Box::new(move |input| {
        let (input, tag) = tag::parse(input)?;
        let (input, children) = parse_nodes(depth + 1)(input)?;

        Ok((input, Node::Element { tag, children }))
    })
}

fn parse_node(depth: usize) -> Box<dyn Fn(&str) -> IResult<&str, Node>> {
    Box::new(move |input| {
        let (input, _) = count(tag("  "), depth)(input)?;
        alt((
            parse_markdown(depth),
            parse_for_loop_with_index(depth),
            parse_for_loop(depth),
            parse_if_else(depth),
            parse_if(depth),
            parse_block_contents(depth),
            parse_fragment,
            parse_node_with_text(depth),
            parse_node_with_interpolated_text(depth),
            parse_node_without_text(depth),
            parse_text_node,
        ))(input)
    })
}

fn parse_fragment_subclass(input: &str) -> IResult<&str, Nodes> {
    let (input, layout) = terminated(parse_extends, tag("\n"))(input)?;
    let (input, blocks) = separated_list1(
        tag("\n"),
        preceded(many0(tag("\n")), parse_block_contents(0)),
    )(input)?;
    let (input, _) = many0(tag("\n"))(input)?;

    Ok((input, Nodes::new_fragment_subclass(layout, blocks)))
}

pub fn to_newline(input: &str) -> IResult<&str, &str> {
    take_till(|c| c == '\n')(input)
}

#[cfg(test)]
mod tests {
    use super::super::{context::*, Socket};

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
    fn div_with_tailwind_classes() {
        assert_eq!(
            Socket::parse(".custom-class.other.lg:max-w-sm.w-1/4")
                .unwrap()
                .to_html(),
            "<div class=\"custom-class other lg:max-w-sm w-1/4\"></div>"
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
                .with_context(build_context("{\"title\": \"Hello world\"}"))
                .map(|v| v.to_html())
                .unwrap(),
            "<h1>Hello world</h1>"
        );
    }

    #[test]
    fn object_interpolation() {
        assert_eq!(
            Socket::parse("%h1= title.primary\n%h2= title.secondary")
                .unwrap()
                .with_context(build_context("{\"title\": {\"primary\": \"Hello world\", \"secondary\": \"wow this works\"}}"))
                .map(|v| v.to_html())
                .unwrap(),
            "<h1>Hello world</h1><h2>wow this works</h2>"
        );
    }

    #[test]
    fn attribute_interpolation() {
        assert_eq!(
            Socket::parse("%a(href=mailto:{contact.email})= contact.name")
                .unwrap()
                .with_context(build_context("{\"contact\": {\"email\": \"person@example.com\", \"name\": \"Person's name\"}}"))
                .map(|v| v.to_html())
                .unwrap(),
            "<a href=\"mailto:person@example.com\">Person's name</a>"
        );
    }

    #[test]
    fn for_loops() {
        assert_eq!(
            Socket::parse("%ul\n  - for value in values\n    %li= value")
                .unwrap()
                .with_context(build_context(
                    "{\"values\": [\"first\", \"second\", \"third\"]}"
                ))
                .map(|v| v.to_html())
                .unwrap(),
            "<ul><li>first</li><li>second</li><li>third</li></ul>"
        );
    }

    #[test]
    fn if_statement() {
        assert_eq!(
            Socket::parse("- if flag\n  %p= value")
                .unwrap()
                .with_context(build_context("{\"value\": \"hello\", \"flag\": true}"))
                .map(|v| v.to_html())
                .unwrap(),
            "<p>hello</p>"
        );
        assert_eq!(
            Socket::parse("- if flag\n  %p= value")
                .unwrap()
                .with_context(build_context("{\"value\": \"hello\", \"flag\": false}"))
                .map(|v| v.to_html())
                .unwrap(),
            ""
        );
    }

    #[test]
    fn if_else_statement() {
        assert_eq!(
            Socket::parse("- if flag\n  %p= value\n- else\n  %p= other")
                .unwrap()
                .with_context(build_context(
                    "{\"value\": \"hello\", \"other\": \"good bye\", \"flag\": false}"
                ))
                .map(|v| v.to_html())
                .unwrap(),
            "<p>good bye</p>"
        );
    }

    #[test]
    fn nested_if_statement() {
        assert_eq!(
            Socket::parse("- if flag\n  - if otherflag\n    %p= works\n- else\n  %p= works")
                .unwrap()
                .with_context(build_context(
                    "{\"works\": \"sure does\", \"otherflag\": false, \"flag\": true}"
                ))
                .map(|v| v.to_html())
                .unwrap(),
            ""
        );

        assert_eq!(
            Socket::parse(
                r#"
- if flag
  - if otherflag
    %p= works

  - else
    %p do not get here"#
            )
            .unwrap()
            .with_context(build_context(
                "{\"works\": \"sure does\", \"otherflag\": true, \"flag\": true}"
            ))
            .map(|v| v.to_html())
            .unwrap(),
            "<p>sure does</p>"
        );
    }

    #[test]
    fn for_loops_with_object() {
        assert_eq!(
            Socket::parse("%ul\n  - for value in values\n    %li= value.name")
                .unwrap()
                .with_context(build_context(
                    "{\"values\": [{\"name\": \"Jane\"}, {\"name\": \"John\"}]}"
                ))
                .map(|v| v.to_html())
                .unwrap(),
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
            .with_context(build_context(
                "{\"items\": [{\"name\": \"Jane\"}, {\"name\": \"John\"}]}"
            ))
            .map(|v| v.to_html())
            .unwrap(),
            "<ul><li>Jane</li><li>Separator</li><li>John</li><li>Separator</li></ul>",
        )
    }

    #[test]
    fn extends_and_blocks() {
        use std::collections::HashMap;
        use std::path::PathBuf;

        let mut fragments: HashMap<PathBuf, String> = HashMap::new();
        fragments.insert(
            PathBuf::from("foo/page.skt"),
            ".foo\n  %h2= block page-header\n  - block contents".into(),
        );

        fragments.insert(
            PathBuf::from("foo/included.skt"),
            "- extends foo/page.skt\n- block page-header\n  Hello world\n\n\n\n- block contents\n  %p Hi\n  %p Hello\n\n\n".into(),
        );

        assert_eq!(
            Socket::parse("%section\n  - fragment foo/included.skt")
                .unwrap()
                .with_fragments(&fragments)
                .to_html(),
            "<section><div class=\"foo\"><h2>Hello world</h2><p>Hi</p><p>Hello</p></div></section>",
        )
    }

    #[test]
    fn markdown_support() {
        assert_eq!(
            Socket::parse(
                ".markdown-text\n  :markdown\n\n    # hello world!\n\n    ## hey\n.other\n  :markdown\n    hi\n\n    hello\n    * first\n    * second"
            )
            .unwrap()
            .to_html(),
            "<div class=\"markdown-text\"><h1>hello world!</h1>\n<h2>hey</h2>\n</div><div class=\"other\"><p>hi</p>\n<p>hello</p>\n<ul>\n<li>\n<p>first</p>\n</li>\n<li>\n<p>second</p>\n</li>\n</ul>\n</div>"
        )
    }

    fn build_context(input: &str) -> Option<Result<Context, ContextError>> {
        Some(Context::load(input))
    }
}
