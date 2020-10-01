use super::{context::Context, flags::Flags, fragments, styles, Socket};
use std::io::{self, Read};
use std::path::PathBuf;
use structopt::StructOpt;

pub fn run() {
    let flags = Flags::from_args();
    let context = flags.context.map(Context::from_file);
    let scss_entrypoint = PathBuf::from("styles/app.scss");

    match read_from_stdin() {
        Ok(input) => {
            if let Ok(mut parsed) = Socket::parse(&input) {
                println!(
                    "{}",
                    parsed
                        .with_fragments(&fragments::new())
                        .with_styles(styles::generate(scss_entrypoint))
                        .with_context(context)
                        .map(|v| v.to_html())
                        .unwrap_or("".to_string())
                );
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
