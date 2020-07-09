use super::{
    context::{Context, Selector},
    Blocks, Fragments, Nodes, Tag,
};
use serde_json::Value;
use std::path::PathBuf;

pub enum Node<'a> {
    Text(&'a str),
    InterpolatedText(Vec<Selector<'a>>),
    BlockValue(&'a str),
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
    Block {
        name: &'a str,
        children: Nodes<'a>,
    },
}

impl<'a> Node<'a> {
    pub fn to_html(
        &self,
        context: &Context,
        fragments: &Fragments<'a>,
        blocks: &Blocks<'a>,
        styles: &'a Option<String>,
    ) -> String {
        match self {
            Node::Text(v) => v.to_string(),
            Node::InterpolatedText(v) => context.interpret(v),
            Node::BlockValue(name) => {
                if let Some(boxed_nodes) = blocks.get(name) {
                    (*boxed_nodes.to_html(context, fragments, blocks, styles)).to_string()
                } else {
                    "".into()
                }
            }
            Node::Element { tag, children } => format!(
                "{}{}{}{}",
                tag.open_tag_html(context),
                children.to_html(context, fragments, blocks, styles),
                tag.additional_markup(styles),
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
                            children.to_html(
                                &context.extend(local, looped_value),
                                fragments,
                                blocks,
                                styles,
                            )
                        })
                        .collect::<Vec<String>>()
                        .join("")
                } else {
                    "".into()
                }
            }
            Node::Fragment { path } => {
                if let Some(nodes) = fragments.get(path) {
                    nodes.to_html(context, fragments, blocks, styles)
                } else {
                    "".into()
                }
            }
            Node::Block { name, children } => {
                if let Some(boxed_nodes) = blocks.get(name) {
                    (*boxed_nodes.to_html(context, fragments, blocks, styles)).to_string()
                } else {
                    children.to_html(context, fragments, blocks, styles)
                }
            }
        }
    }
}
