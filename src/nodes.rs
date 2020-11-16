use super::{context::Context, Builder, Fragments, Node, NodeError};
use std::collections::HashMap;
use std::path::PathBuf;

pub type Blocks<'a> = HashMap<&'a str, Box<Nodes<'a>>>;

pub enum Nodes<'a> {
    Fragment { nodes: Vec<Node<'a>> },
    Document { nodes: Vec<Node<'a>> },
    FragmentSubclass { layout: PathBuf, blocks: Blocks<'a> },
}

impl<'a> Default for Nodes<'a> {
    fn default() -> Self {
        Nodes::Fragment {
            nodes: Vec::default(),
        }
    }
}

impl<'a> Nodes<'a> {
    pub fn to_html(
        &self,
        mut builder: Builder<String, NodeError<'a>>,
        context: &Context,
        fragments: &Fragments<'a>,
        blocks: &Blocks<'a>,
        styles: &'a Option<String>,
    ) -> Builder<String, NodeError<'a>> {
        match self {
            Nodes::Fragment { nodes } => {
                builder = Self::nodes_to_html(builder, nodes, context, fragments, blocks, styles);
            }
            Nodes::Document { nodes } => {
                builder.append("<!DOCTYPE html>".to_string());
                builder = Self::nodes_to_html(builder, nodes, context, fragments, blocks, styles);
            }
            Nodes::FragmentSubclass { layout, blocks } => {
                if let Some(nodes) = fragments.get(layout) {
                    builder = nodes.to_html(builder, context, fragments, blocks, styles)
                } else {
                    builder.warn(NodeError::InvalidFragmentPath(layout.to_path_buf()))
                }
            }
        }

        builder
    }

    pub fn prepend(&mut self, node: Node<'a>) {
        match self {
            Nodes::Fragment { nodes } => nodes.insert(0, node),
            Nodes::Document { nodes } => nodes.insert(0, node),
            Nodes::FragmentSubclass { .. } => (),
        }
    }

    pub fn new_fragment(nodes: Vec<Node<'a>>) -> Self {
        Nodes::Fragment { nodes }
    }

    pub fn new_document(nodes: Vec<Node<'a>>) -> Self {
        Nodes::Document { nodes }
    }

    pub fn new_fragment_subclass(layout: PathBuf, nodes: Vec<Node<'a>>) -> Self {
        let mut blocks = HashMap::new();
        for node in nodes {
            if let Node::Block { name, children } = node {
                blocks.insert(name, Box::new(children));
            }
        }

        Nodes::FragmentSubclass { layout, blocks }
    }

    fn nodes_to_html(
        builder: Builder<String, NodeError<'a>>,
        nodes: &[Node<'a>],
        context: &Context,
        fragments: &Fragments<'a>,
        blocks: &Blocks<'a>,
        styles: &'a Option<String>,
    ) -> Builder<String, NodeError<'a>> {
        nodes.iter().fold(builder, |acc, n| {
            n.to_html(acc, context, fragments, blocks, styles)
        })
    }
}
