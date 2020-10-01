use sass_rs::{Options as SassOptions, OutputStyle};
use std::path::Path;

#[derive(Debug)]
pub struct SassCompileError(String);

pub fn generate<P: AsRef<Path>>(path: P) -> Result<String, SassCompileError> {
    let mut options = SassOptions::default();
    options.output_style = OutputStyle::Compressed;

    sass_rs::compile_file(path, options).map_err(SassCompileError)
}

pub enum Styles {
    NotProcessed,
    StyleError(SassCompileError),
    Styles(String),
}

impl std::default::Default for Styles {
    fn default() -> Self {
        Styles::NotProcessed
    }
}

impl Styles {
    pub fn as_option(&self) -> Option<String> {
        match self {
            Styles::Styles(v) => Some(v.to_string()),
            _ => None,
        }
    }
}

impl From<Result<String, SassCompileError>> for Styles {
    fn from(result: Result<String, SassCompileError>) -> Self {
        match result {
            Ok(v) => Styles::Styles(v),
            Err(e) => Styles::StyleError(e),
        }
    }
}
