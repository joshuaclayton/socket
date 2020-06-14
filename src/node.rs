use super::{
    context::{Context, Selector},
    Nodes, Tag,
};

pub enum Node<'a> {
    Text(&'a str),
    InterpolatedText(Vec<Selector<'a>>),
    Element { tag: Tag<'a>, children: Nodes<'a> },
}

impl<'a> Node<'a> {
    pub fn to_html(&self, context: &Context) -> String {
        match self {
            Node::Text(v) => v.to_string(),
            Node::InterpolatedText(v) => context.interpret(v.iter()),
            Node::Element { tag, children } => format!(
                "{}{}{}",
                tag.open_tag_html(),
                children.to_html(context),
                tag.close_tag_html()
            ),
        }
    }
}
