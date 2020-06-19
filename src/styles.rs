use sass_rs::{Options as SassOptions, OutputStyle};
use std::path::PathBuf;

pub fn generate() -> Option<String> {
    let mut options = SassOptions::default();
    options.output_style = OutputStyle::Compressed;
    let path = PathBuf::from("styles/app.scss");

    sass_rs::compile_file(path, options).ok()
}
