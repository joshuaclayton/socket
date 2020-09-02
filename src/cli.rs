use super::{flags::Flags, fragments, styles, Socket};
use std::fs;
use std::io::{self, Read};
use std::path::PathBuf;
use structopt::StructOpt;

pub fn run() {
    let flags = Flags::from_args();
    let context = flags.context.and_then(|v| read_file(&v).ok());
    let scss_entrypoint = PathBuf::from("styles/app.scss");

    match read_from_stdin() {
        Ok(input) => {
            if let Ok(mut parsed) = Socket::parse(&input) {
                match context {
                    None => println!(
                        "{}",
                        parsed
                            .with_fragments(&fragments::new())
                            .with_styles(styles::generate(scss_entrypoint))
                            .map(|v| v.to_html())
                            .unwrap_or("".to_string())
                    ),
                    Some(ctx) => println!(
                        "{}",
                        parsed
                            .with_fragments(&fragments::new())
                            .with_styles(styles::generate(scss_entrypoint))
                            .and_then(|v| v.with_context(&ctx))
                            .map(|v| v.to_html())
                            .unwrap_or("".to_string())
                    ),
                }
            } else {
                eprintln!("Unable to parse input");
                std::process::exit(1)
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
