use super::{
    context::{Context, ContextError},
    parser, styles, Builder, Nodes, Styles,
};
use std::collections::HashMap;
use std::path::PathBuf;

pub type Fragments<'a> = HashMap<PathBuf, Nodes<'a>>;

pub struct Socket<'a> {
    nodes: Nodes<'a>,
    context: Context,
    fragments: Result<Fragments<'a>, Vec<FragmentError<'a>>>,
    styles: Styles,
}

#[derive(Debug)]
pub enum FragmentError<'a> {
    IncompleteParse(PathBuf),
    ParseError(nom::Err<(&'a str, nom::error::ErrorKind)>),
}

#[derive(Debug)]
pub enum SocketError<'a> {
    FragmentError(FragmentError<'a>),
    ParseError(nom::Err<(&'a str, nom::error::ErrorKind)>),
    StyleError(styles::SassCompileError),
    ContextError(ContextError),
}

impl<'a> Socket<'a> {
    pub fn parse(input: &str) -> Result<Socket, SocketError> {
        let (_, nodes) = parser::parse(input).map_err(SocketError::ParseError)?;
        let context = Context::empty();
        let fragments = Ok(HashMap::new());
        Ok(Socket {
            nodes,
            context,
            fragments,
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
        let fragments: Builder<_, _> = frags
            .iter()
            .map(|(k, v)| match parser::parse(v) {
                Ok(("", n)) => Ok((k.to_path_buf(), n)),
                Ok(_) => Err(FragmentError::IncompleteParse(k.to_path_buf())),
                Err(e) => Err(FragmentError::ParseError(e)),
            })
            .collect::<Vec<_>>()
            .into();

        let result: Result<_, _> = fragments.into();

        self.fragments = result.map(|v| v.into_iter().collect::<HashMap<_, _>>());

        self
    }

    pub fn to_html(&self) -> String {
        self.nodes
            .to_html(
                Builder::default(),
                &self.context,
                &self.fragments.as_ref().unwrap_or(&HashMap::new()),
                &HashMap::new(),
                &self.styles.as_option(),
            )
            .result()
            .join("")
    }
}
