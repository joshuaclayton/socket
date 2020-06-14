use super::{Nodes, Tag};

pub enum Node<'a> {
    Text(&'a str),
    Element { tag: Tag<'a>, children: Nodes<'a> },
}

impl<'a> Node<'a> {
    pub fn to_html(&self) -> String {
        match self {
            Node::Text(v) => v.to_string(),
            Node::Element { tag, children } => format!(
                "{}{}{}",
                tag.open_tag_html(),
                children.to_html(),
                tag.close_tag_html()
            ),
        }
    }
}
