use super::{context::Context, parser, Nodes};

pub struct Socket<'a> {
    nodes: Nodes<'a>,
    context: Context,
}

impl<'a> Socket<'a> {
    pub fn parse(input: &str) -> Result<Socket, nom::Err<(&str, nom::error::ErrorKind)>> {
        let (_, nodes) = parser::parse(input)?;
        let context = Context::empty();
        Ok(Socket { nodes, context })
    }

    pub fn with_context(&mut self, context: &str) -> &mut Self {
        self.context = Context::load(context);
        self
    }

    pub fn to_html(&self) -> String {
        self.nodes.to_html(&self.context)
    }
}
