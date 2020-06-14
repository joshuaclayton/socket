use super::Socket;
use std::io::{self, Read};

pub fn run() {
    match read_from_stdin() {
        Ok(input) => {
            if let Ok(parsed) = Socket::parse(&input) {
                println!("{}", parsed.to_html());
            }
        }
        Err(e) => {
            eprintln!("{}", e);
            std::process::exit(1)
        }
    }
}

fn read_from_stdin() -> io::Result<String> {
    let mut buffer = String::new();
    io::stdin().read_to_string(&mut buffer)?;
    Ok(buffer)
}
