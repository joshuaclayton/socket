use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

pub fn new() -> HashMap<PathBuf, String> {
    let mut map = HashMap::new();

    for entry in WalkDir::new(".")
        .follow_links(true)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let path = entry.path().strip_prefix("./").unwrap();

        if is_fragment(path) {
            if let Some(file_body) = read_file(path) {
                map.insert(path.into(), file_body);
            }
        };
    }

    map
}

fn read_file<P: AsRef<Path>>(filename: P) -> Option<String> {
    fs::read_to_string(filename).ok()
}

fn is_fragment(path: &Path) -> bool {
    path.starts_with("fragments") && path.extension() == Some(std::ffi::OsStr::new("skt"))
}
