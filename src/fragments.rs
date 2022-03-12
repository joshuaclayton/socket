use super::{parser, Nodes};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

pub struct Fragments<'a>(HashMap<PathBuf, Result<Nodes<'a>, FragmentError<'a>>>);

impl<'a> Fragments<'a> {
    pub fn get(&self, key: &PathBuf) -> Option<&Nodes<'a>> {
        match self.0.get(key) {
            Some(Ok(v)) => Some(v),
            _ => None,
        }
    }

    pub fn insert(
        &mut self,
        key: PathBuf,
        value: Result<Nodes<'a>, FragmentError<'a>>,
    ) -> &mut Self {
        self.0.insert(key, value);
        self
    }

    pub fn parse(path: &PathBuf, input: &'a str) -> Result<Nodes<'a>, FragmentError<'a>> {
        match parser::parse(input) {
            Ok(("", n)) => Ok(n),
            Ok(_) => Err(FragmentError::IncompleteParse(path.to_path_buf())),
            Err(e) => Err(FragmentError::ParseError(e)),
        }
    }
}

impl<'a> std::default::Default for Fragments<'a> {
    fn default() -> Self {
        Fragments(HashMap::new())
    }
}

#[derive(Debug)]
pub enum FragmentError<'a> {
    IncompleteParse(PathBuf),
    ParseError(nom::Err<(&'a str, nom::error::ErrorKind)>),
}

pub fn new(fragments_path: PathBuf) -> HashMap<PathBuf, String> {
    let mut map = HashMap::new();

    for entry in WalkDir::new(fragments_path.clone())
        .follow_links(true)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let path = entry.path().strip_prefix(&fragments_path);

        if is_fragment(&path) {
            if let Some(file_body) = read_file(entry.path()) {
                map.insert(path.unwrap().into(), file_body);
            }
        };
    }

    map
}

fn read_file<P: AsRef<Path>>(filename: P) -> Option<String> {
    fs::read_to_string(filename).ok()
}

fn is_fragment<T>(path: &Result<&Path, T>) -> bool {
    match path {
        Ok(v) => v.extension() == Some(std::ffi::OsStr::new("skt")),
        _ => false,
    }
}
