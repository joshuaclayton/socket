use super::{context::Context, parser, Nodes};
use std::collections::HashMap;
use std::path::PathBuf;

pub type Fragments<'a> = HashMap<PathBuf, Nodes<'a>>;

pub struct Socket<'a> {
    nodes: Nodes<'a>,
    context: Context,
    fragments: Fragments<'a>,
    styles: &'a Option<String>,
}

impl<'a> Socket<'a> {
    pub fn parse(input: &str) -> Result<Socket, nom::Err<(&str, nom::error::ErrorKind)>> {
        let (_, nodes) = parser::parse(input)?;
        let context = Context::empty();
        let fragments = HashMap::new();
        Ok(Socket {
            nodes,
            context,
            fragments,
            styles: &None,
        })
    }

    pub fn with_context(&mut self, context: &str) -> &mut Self {
        self.context = Context::load(context);
        self
    }

    pub fn with_styles(&mut self, styles: &'a Option<String>) -> &mut Self {
        self.styles = styles;
        self
    }

    pub fn with_fragments(&mut self, frags: &'a HashMap<PathBuf, String>) -> &mut Self {
        let fragments = frags
            .iter()
            .filter_map(|(k, v)| match parser::parse(v).ok() {
                Some(("", n)) => Some((k.to_path_buf(), n)),
                _ => None,
            })
            .collect::<HashMap<_, _>>();
        self.fragments = fragments;
        self
    }

    pub fn to_html(&self) -> String {
        self.nodes
            .to_html(&self.context, &self.fragments, &self.styles)
    }
}
