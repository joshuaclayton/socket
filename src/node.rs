use super::{
    context::{Context, Selector},
    Blocks, Builder, CompiledNodes, Fragments, Nodes, Tag,
};
use pulldown_cmark::{html, Options, Parser};
use serde_json::Value;
use std::path::PathBuf;

pub enum Node<'a> {
    Text(&'a str),
    Markdown(Vec<&'a str>),
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
    IfElse {
        selectors: Vec<Selector<'a>>,
        true_children: Nodes<'a>,
        false_children: Nodes<'a>,
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
pub enum CompiledNode<'a> {
    Text(&'a str),
    Markdown(String),
    InterpolatedText(&'a Vec<Selector<'a>>),
    Nodes(CompiledNodes<'a>),
    Element {
        tag: &'a Tag<'a>,
        children: CompiledNodes<'a>,
    },
    ForLoop {
        local: &'a str,
        selectors: &'a Vec<Selector<'a>>,
        children: CompiledNodes<'a>,
    },
    IfElse {
        selectors: &'a Vec<Selector<'a>>,
        true_children: CompiledNodes<'a>,
        false_children: CompiledNodes<'a>,
    },
}

#[derive(Debug)]
pub enum NodeError<'a> {
    InvalidFragmentPath(PathBuf),
    InvalidBlockName(&'a str),
    JSONValueMissingAtSelector(Vec<Selector<'a>>),
    JSONValueNotArrayAtSelector(Vec<Selector<'a>>),
    JSONValueNotBoolAtSelector(Vec<Selector<'a>>),
}

impl<'a> Node<'a> {
    pub fn compile(
        &'a self,
        blocks: &'a Blocks<'a>,
        fragments: &'a Fragments<'a>,
    ) -> Result<CompiledNode<'a>, Vec<NodeError<'a>>> {
        match self {
            Node::Text(v) => Ok(CompiledNode::Text(v)),
            Node::Markdown(lines) => {
                let result = lines.join("\n\n");
                let parser = Parser::new_ext(&result, Options::empty());
                let mut html_output = String::new();
                html::push_html(&mut html_output, parser);

                Ok(CompiledNode::Markdown(html_output))
            }
            Node::InterpolatedText(v) => Ok(CompiledNode::InterpolatedText(v)),
            Node::BlockValue(name) => blocks
                .get(name)
                .ok_or(vec![NodeError::InvalidBlockName(name)])
                .and_then(|nodes| {
                    nodes
                        .compile(blocks, fragments)
                        .map(|nodes| CompiledNode::Nodes(nodes))
                }),
            Node::Element { tag, children } => children
                .compile(blocks, fragments)
                .map(|children| CompiledNode::Element { tag, children }),
            Node::ForLoop {
                local,
                selectors,
                children,
            } => children
                .compile(blocks, fragments)
                .map(|children| CompiledNode::ForLoop {
                    local,
                    selectors,
                    children,
                }),
            Node::IfElse {
                selectors,
                true_children,
                false_children,
            } => {
                match (
                    true_children.compile(blocks, fragments),
                    false_children.compile(blocks, fragments),
                ) {
                    (Err(e1), _) => Err(e1),
                    (_, Err(e2)) => Err(e2),
                    (Ok(true_children), Ok(false_children)) => Ok(CompiledNode::IfElse {
                        selectors,
                        true_children,
                        false_children,
                    }),
                }
            }
            Node::Fragment { path } => fragments
                .get(&path)
                .ok_or(vec![NodeError::InvalidFragmentPath(path.to_path_buf())])
                .and_then(|nodes| {
                    nodes
                        .compile(blocks, fragments)
                        .map(|nodes| CompiledNode::Nodes(nodes))
                }),
            Node::Block { name, children } => {
                if let Some(boxed_nodes) = blocks.get(name) {
                    boxed_nodes
                        .compile(blocks, fragments)
                        .map(|nodes| CompiledNode::Nodes(nodes))
                } else {
                    children
                        .compile(blocks, fragments)
                        .map(|nodes| CompiledNode::Nodes(nodes))
                }
            }
        }
    }

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
            Node::Markdown(lines) => {
                let result = lines.join("\n\n");
                let parser = Parser::new_ext(&result, Options::empty());
                let mut html_output = String::new();
                html::push_html(&mut html_output, parser);

                builder.append(html_output)
            }
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
            Node::IfElse {
                selectors,
                true_children,
                false_children,
            } => match context.at(selectors) {
                None => builder.warn(NodeError::JSONValueMissingAtSelector(selectors.to_vec())),
                Some(Value::Bool(true)) => {
                    builder = true_children.to_html(builder, context, fragments, blocks, styles)
                }
                Some(Value::Bool(false)) => {
                    builder = false_children.to_html(builder, context, fragments, blocks, styles)
                }
                Some(_) => builder.warn(NodeError::JSONValueNotBoolAtSelector(selectors.to_vec())),
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

impl<'a> CompiledNode<'a> {
    pub fn to_html(
        &self,
        mut builder: Builder<String, NodeError<'a>>,
        context: &Context,
        styles: &'a Option<String>,
    ) -> Builder<String, NodeError<'a>> {
        match self {
            CompiledNode::Text(v) => builder.append(v.to_string()),
            CompiledNode::Markdown(html_output) => builder.append(html_output.to_string()),
            CompiledNode::InterpolatedText(selectors) => match context.interpret(selectors) {
                None => builder.warn(NodeError::JSONValueMissingAtSelector(selectors.to_vec())),
                Some(value) => builder.append(value),
            },
            CompiledNode::Element { tag, children } => {
                builder.append(tag.open_tag_html(context));
                builder = children.to_html(builder, context, styles);
                builder.append(tag.additional_markup(styles));
                builder.append(tag.close_tag_html());
            }
            CompiledNode::ForLoop {
                local,
                selectors,
                children,
            } => match context.at(selectors) {
                None => builder.warn(NodeError::JSONValueMissingAtSelector(selectors.to_vec())),
                Some(Value::Array(loopable)) => {
                    builder = loopable.iter().fold(builder, |acc, looped_value| {
                        children.to_html(acc, &context.extend(local, looped_value), styles)
                    })
                }
                Some(_) => builder.warn(NodeError::JSONValueNotArrayAtSelector(selectors.to_vec())),
            },
            CompiledNode::IfElse {
                selectors,
                true_children,
                false_children,
            } => match context.at(selectors) {
                None => builder.warn(NodeError::JSONValueMissingAtSelector(selectors.to_vec())),
                Some(Value::Bool(true)) => {
                    builder = true_children.to_html(builder, context, styles)
                }
                Some(Value::Bool(false)) => {
                    builder = false_children.to_html(builder, context, styles)
                }
                Some(_) => builder.warn(NodeError::JSONValueNotBoolAtSelector(selectors.to_vec())),
            },
            CompiledNode::Nodes(nodes) => builder = nodes.to_html(builder, context, styles),
        };

        builder
    }
}
