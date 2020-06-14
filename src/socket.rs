use super::{parser, Nodes};

pub struct Socket<'a> {
    nodes: Nodes<'a>,
}

impl<'a> Socket<'a> {
    pub fn parse(input: &str) -> Result<Socket, nom::Err<(&str, nom::error::ErrorKind)>> {
        let (_, nodes) = parser::parse(input)?;

        Ok(Socket { nodes })
    }

    pub fn to_html(&self) -> String {
        self.nodes.to_html()
    }
}
