use super::{
    context::{Context, Selector},
    Blocks, Builder, Fragments, Nodes, Tag,
};
use pulldown_cmark::{html, Options, Parser};
use serde_json::Value;
use std::collections::BTreeMap;
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
        index: Option<&'a str>,
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
    FragmentWithLocals {
        path: PathBuf,
        locals: BTreeMap<&'a str, Vec<Selector<'a>>>,
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
    JSONValueNotBoolAtSelector(Vec<Selector<'a>>),
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
                index,
                local,
                selectors,
                children,
            } => match context.at(selectors) {
                None => builder.warn(NodeError::JSONValueMissingAtSelector(selectors.to_vec())),
                Some(Value::Array(loopable)) => {
                    let mut idx = 0;
                    builder = loopable.iter().fold(builder, |acc, looped_value| {
                        let mut ctx = context.extend(local, looped_value);

                        if let Some(index) = index {
                            ctx = ctx.extend(index, &Value::Number(idx.into()));
                        }

                        idx += 1;

                        children.to_html(acc, &ctx, fragments, blocks, styles)
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
            Node::FragmentWithLocals { path, locals } => {
                if let Some(nodes) = fragments.get(path) {
                    let mut ctx = context.clone();

                    for (k, v) in locals {
                        if let Some(val) = ctx.at(&v) {
                            ctx = ctx.extend(k, val);
                        }
                    }

                    builder = nodes.to_html(builder, &ctx, fragments, blocks, styles)
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
