use super::{
    context::{Context, ContextError},
    parser, styles, Nodes,
};
use std::collections::HashMap;
use std::path::PathBuf;

pub type Fragments<'a> = HashMap<PathBuf, Nodes<'a>>;

pub struct Socket<'a> {
    nodes: Nodes<'a>,
    context: Context,
    fragments: Fragments<'a>,
    styles: Option<String>,
}

fn hashmap_sequence<K: std::hash::Hash + std::cmp::Eq, V, E>(
    hashmap: Vec<Result<(K, V), E>>,
) -> Result<HashMap<K, V>, Vec<E>> {
    let mut errors = vec![];
    let mut final_map = HashMap::new();

    for item in hashmap {
        match item {
            Err(e) => {
                errors.push(e);
            }
            Ok((k, v)) => {
                final_map.insert(k, v);
            }
        }
    }
    if errors.is_empty() {
        Ok(final_map)
    } else {
        Err(errors)
    }
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
        let fragments = HashMap::new();
        Ok(Socket {
            nodes,
            context,
            fragments,
            styles: None,
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

    pub fn with_styles(
        &mut self,
        styles: Result<String, styles::SassCompileError>,
    ) -> Result<&mut Self, SocketError> {
        match styles {
            Ok(v) => {
                self.styles = Some(v);
                Ok(self)
            }
            Err(e) => Err(SocketError::StyleError(e)),
        }
    }

    pub fn with_fragments(&mut self, frags: &'a HashMap<PathBuf, String>) -> &mut Self {
        let fragments = frags
            .iter()
            .map(|(k, v)| match parser::parse(v) {
                Ok(("", n)) => Ok((k.to_path_buf(), n)),
                Ok(_) => Err(FragmentError::IncompleteParse(k.to_path_buf())),
                Err(e) => Err(FragmentError::ParseError(e)),
            })
            .collect::<Vec<_>>();

        self.fragments = hashmap_sequence(fragments).unwrap_or(HashMap::new());
        self
    }

    pub fn to_html(&self) -> String {
        self.nodes.to_html(
            &self.context,
            &self.fragments,
            &HashMap::new(),
            &self.styles,
        )
    }
}
