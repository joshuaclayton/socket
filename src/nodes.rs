use super::{context::Context, Node};

pub enum Nodes<'a> {
    Fragment { nodes: Vec<Node<'a>> },
    Document { nodes: Vec<Node<'a>> },
}

impl<'a> Nodes<'a> {
    pub fn to_html(&self, context: &Context) -> String {
        match self {
            Nodes::Fragment { nodes } => Self::nodes_to_html(nodes, context),
            Nodes::Document { nodes } => format!(
                "{}{}",
                "<!DOCTYPE html>",
                Self::nodes_to_html(nodes, context)
            ),
        }
    }

    pub fn prepend(&mut self, node: Node<'a>) {
        match self {
            Nodes::Fragment { nodes } => nodes.insert(0, node),
            Nodes::Document { nodes } => nodes.insert(0, node),
        }
    }

    pub fn new_fragment(nodes: Vec<Node<'a>>) -> Self {
        Nodes::Fragment { nodes }
    }

    pub fn new_document(nodes: Vec<Node<'a>>) -> Self {
        Nodes::Document { nodes }
    }

    fn nodes_to_html(nodes: &[Node], context: &Context) -> String {
        nodes
            .iter()
            .map(|n| n.to_html(context))
            .collect::<Vec<String>>()
            .join("")
    }
}
