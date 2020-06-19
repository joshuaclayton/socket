use super::{context::Context, Attribute, Attributes};

pub struct Tag<'a> {
    pub name: &'a str,
    pub attributes: Vec<Attribute<'a>>,
}

impl<'a> Tag<'a> {
    pub fn open_tag_html(&self, context: &Context) -> String {
        let attributes: Attributes = self.attributes.clone().into();

        match attributes.to_html(context).as_slice() {
            [] => format!("<{}>", self.name),
            attrs => format!("<{} {}>", self.name, attrs.join(" ")),
        }
    }

    pub fn close_tag_html(&self) -> String {
        format!("</{}>", self.name)
    }

    pub fn additional_markup(&self, styles: &Option<String>) -> String {
        match (self.name, styles) {
            ("head", Some(v)) => format!("<style>\n{}</style>", v),
            _ => "".into(),
        }
    }
}
