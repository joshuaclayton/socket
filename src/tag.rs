use super::{Attribute, Attributes};

pub struct Tag<'a> {
    pub name: &'a str,
    pub attributes: Vec<Attribute<'a>>,
}

impl<'a> Tag<'a> {
    pub fn open_tag_html(&self) -> String {
        let attributes: Attributes = self.attributes.clone().into();

        if attributes.to_html().is_empty() {
            format!("<{}>", self.name)
        } else {
            format!("<{} {}>", self.name, attributes.to_html().join(" "))
        }
    }

    pub fn close_tag_html(&self) -> String {
        format!("</{}>", self.name)
    }
}
