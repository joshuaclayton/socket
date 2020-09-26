use super::{
    context::{Context, Selector},
    Blocks, Builder, Fragments, Nodes, Tag,
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

#[derive(Debug)]
pub enum NodeError<'a> {
    InvalidFragmentPath(PathBuf),
    InvalidBlockName(&'a str),
    JSONValueMissingAtSelector(Vec<Selector<'a>>),
    JSONValueNotArrayAtSelector(Vec<Selector<'a>>),
}

impl<'a> Node<'a> {
    pub fn to_html(
        &self,
        mut builder: Builder<String, NodeError<'a>>,
        context: &Context,
        fragments: &Fragments<'a>,
        blocks: &Blocks<'a>,
        styles: &'a Option<String>,
    ) -> Builder<String, NodeError<'a>> {
        match self {
            Node::Text(v) => builder.append(v.to_string()),
            Node::InterpolatedText(selectors) => match context.interpret(selectors) {
                None => builder.warn(NodeError::JSONValueMissingAtSelector(selectors.to_vec())),
                Some(value) => builder.append(value),
            },
            Node::BlockValue(name) => {
                if let Some(boxed_nodes) = blocks.get(name) {
                    builder = boxed_nodes.to_html(builder, context, fragments, blocks, styles);
                } else {
                    builder.warn(NodeError::InvalidBlockName(name))
                }
            }
            Node::Element { tag, children } => {
                builder.append(tag.open_tag_html(context));
                builder = children.to_html(builder, context, fragments, blocks, styles);
                builder.append(tag.additional_markup(styles));
                builder.append(tag.close_tag_html());
            }
            Node::ForLoop {
                local,
                selectors,
                children,
            } => match context.at(selectors) {
                None => builder.warn(NodeError::JSONValueMissingAtSelector(selectors.to_vec())),
                Some(Value::Array(loopable)) => {
                    builder = loopable.iter().fold(builder, |acc, looped_value| {
                        children.to_html(
                            acc,
                            &context.extend(local, looped_value),
                            fragments,
                            blocks,
                            styles,
                        )
                    })
                }
                Some(_) => builder.warn(NodeError::JSONValueNotArrayAtSelector(selectors.to_vec())),
            },
            Node::Fragment { path } => {
                if let Some(nodes) = fragments.get(path) {
                    builder = nodes.to_html(builder, context, fragments, blocks, styles)
                } else {
                    builder.warn(NodeError::InvalidFragmentPath(path.to_path_buf()))
                }
            }
            Node::Block { name, children } => {
                if let Some(boxed_nodes) = blocks.get(name) {
                    builder = boxed_nodes.to_html(builder, context, fragments, blocks, styles)
                } else {
                    builder = children.to_html(builder, context, fragments, blocks, styles)
                }
            }
        };

        builder
    }
}
