use super::{context::Context, Fragments, Node};
use std::collections::HashMap;
use std::path::PathBuf;

pub type Blocks<'a> = HashMap<&'a str, Box<Nodes<'a>>>;

pub enum Nodes<'a> {
    Fragment { nodes: Vec<Node<'a>> },
    Document { nodes: Vec<Node<'a>> },
    FragmentSubclass { layout: PathBuf, blocks: Blocks<'a> },
}

impl<'a> Nodes<'a> {
    pub fn to_html(
        &self,
        context: &Context,
        fragments: &Fragments<'a>,
        blocks: &Blocks<'a>,
        styles: &'a Option<String>,
    ) -> String {
        match self {
            Nodes::Fragment { nodes } => {
                Self::nodes_to_html(nodes, context, fragments, blocks, styles)
            }
            Nodes::Document { nodes } => format!(
                "{}{}",
                "<!DOCTYPE html>",
                Self::nodes_to_html(nodes, context, fragments, blocks, styles)
            ),
            Nodes::FragmentSubclass { layout, blocks } => {
                if let Some(nodes) = fragments.get(layout) {
                    nodes.to_html(context, fragments, blocks, styles)
                } else {
                    "".into()
                }
            }
        }
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
        nodes: &[Node],
        context: &Context,
        fragments: &Fragments<'a>,
        blocks: &Blocks<'a>,
        styles: &'a Option<String>,
    ) -> String {
        nodes
            .iter()
            .map(|n| n.to_html(context, fragments, blocks, styles))
            .collect::<Vec<String>>()
            .join("")
    }
}
