use super::{
    context::{Context, ContextError},
    fragments::Fragments,
    parser, styles, Blocks, Builder, CompiledNodes, NodeError, Nodes, Styles,
};
use std::collections::HashMap;
use std::path::PathBuf;

pub struct Socket<'a> {
    nodes: Nodes<'a>,
    context: Context,
    fragments: Fragments<'a>,
    blocks: Blocks<'a>,
    styles: Styles,
}

#[derive(Debug)]
pub enum SocketError<'a> {
    ParseError(nom::Err<(&'a str, nom::error::ErrorKind)>),
    StyleError(styles::SassCompileError),
    ContextError(ContextError),
}

impl<'a> Socket<'a> {
    pub fn parse(input: &str) -> Result<Socket, SocketError> {
        let (_, nodes) = parser::parse(input).map_err(SocketError::ParseError)?;
        let context = Context::empty();
        let fragments = Fragments::default();
        let blocks = Blocks::default();
        Ok(Socket {
            nodes,
            context,
            fragments,
            blocks,
            styles: Styles::default(),
        })
    }

    pub fn with_context(
        &mut self,
        context: Option<Result<Context, ContextError>>,
    ) -> Result<&mut Self, SocketError> {
        if let Some(context_) = context {
            match context_ {
                Ok(v) => {
                    self.context = v;
                    Ok(self)
                }
                Err(e) => Err(SocketError::ContextError(e)),
            }
        } else {
            Ok(self)
        }
    }

    pub fn with_styles<T: Into<Styles>>(&mut self, styles: T) -> &mut Self {
        self.styles = styles.into();
        self
    }

    pub fn with_fragments(&mut self, frags: &'a HashMap<PathBuf, String>) -> &mut Self {
        for (k, v) in frags {
            self.fragments
                .insert(k.to_path_buf(), Fragments::parse(k, v));
        }

        self
    }

    pub fn compile(&'a self) -> Result<CompiledNodes<'a>, Vec<NodeError<'a>>> {
        self.nodes.compile(&self.blocks, &self.fragments)
    }

    pub fn to_html(&self) -> String {
        self.nodes
            .to_html(
                Builder::default(),
                &self.context,
                &self.fragments,
                &HashMap::new(),
                &self.styles.as_option(),
            )
            .result()
            .join("")
    }
}
