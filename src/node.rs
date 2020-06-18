use super::{
    context::{Context, Selector},
    Fragments, Nodes, Tag,
};
use serde_json::Value;
use std::path::PathBuf;

pub enum Node<'a> {
    Text(&'a str),
    InterpolatedText(Vec<Selector<'a>>),
    Element {
        tag: Tag<'a>,
        children: Nodes<'a>,
    },
    ForLoop {
        local: &'a str,
        selectors: Vec<Selector<'a>>,
        children: Nodes<'a>,
    },
    Fragment {
        path: PathBuf,
    },
}

impl<'a> Node<'a> {
    pub fn to_html(&self, context: &Context, fragments: &Fragments<'a>) -> String {
        match self {
            Node::Text(v) => v.to_string(),
            Node::InterpolatedText(v) => context.interpret(v),
            Node::Element { tag, children } => format!(
                "{}{}{}",
                tag.open_tag_html(),
                children.to_html(context, fragments),
                tag.close_tag_html()
            ),
            Node::ForLoop {
                local,
                selectors,
                children,
            } => {
                if let Some(Value::Array(loopable)) = context.at(selectors) {
                    loopable
                        .iter()
                        .map(|looped_value| {
                            children.to_html(&context.extend(local, looped_value), fragments)
                        })
                        .collect::<Vec<String>>()
                        .join("")
                } else {
                    "".into()
                }
            }
            Node::Fragment { path } => {
                if let Some(nodes) = fragments.get(path) {
                    nodes.to_html(context, fragments)
                } else {
                    "".into()
                }
            }
        }
    }
}
