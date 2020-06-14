use std::path::PathBuf;
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
#[structopt(name = "socket")]
pub struct Flags {
    /// Context file
    ///
    /// Load context from a JSON file
    #[structopt(long)]
    pub context: Option<PathBuf>,
}
