use sass_rs::{Options as SassOptions, OutputStyle};
use std::path::Path;

#[derive(Debug)]
pub struct SassCompileError(String);

pub fn generate<P: AsRef<Path>>(path: P) -> Result<String, SassCompileError> {
    let mut options = SassOptions::default();
    options.output_style = OutputStyle::Compressed;

    sass_rs::compile_file(path, options).map_err(SassCompileError)
}
