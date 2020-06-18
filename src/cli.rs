use super::{flags::Flags, fragments, Socket};
use std::fs;
use std::io::{self, Read};
use std::path::PathBuf;
use structopt::StructOpt;

pub fn run() {
    let flags = Flags::from_args();
    let context = flags.context.and_then(|v| read_file(&v).ok());

    match read_from_stdin() {
        Ok(input) => {
            if let Ok(mut parsed) = Socket::parse(&input) {
                match context {
                    None => println!("{}", parsed.to_html()),
                    Some(ctx) => println!(
                        "{}",
                        parsed
                            .with_fragments(&fragments::new())
                            .with_context(&ctx)
                            .to_html()
                    ),
                }
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

fn read_file(filename: &PathBuf) -> Result<String, io::Error> {
    let contents = fs::read_to_string(filename)?;

    Ok(contents)
}
