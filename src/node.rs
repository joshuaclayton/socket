use super::{
    context::{Context, Selector},
    Nodes, Tag,
};
use serde_json::Value;

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
}

impl<'a> Node<'a> {
    pub fn to_html(&self, context: &Context) -> String {
        match self {
            Node::Text(v) => v.to_string(),
            Node::InterpolatedText(v) => context.interpret(v),
            Node::Element { tag, children } => format!(
                "{}{}{}",
                tag.open_tag_html(),
                children.to_html(context),
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
                        .map(|looped_value| children.to_html(&context.extend(local, looped_value)))
                        .collect::<Vec<String>>()
                        .join("")
                } else {
                    "".into()
                }
            }
        }
    }
}
