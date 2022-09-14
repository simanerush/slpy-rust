//! The Rust implementation of slpy.
use clap::Parser;
use slpy_rust::{dump, repl, Result};

/// The slpy programming language.
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// Name of the person to greet
    #[clap(short, long)]
    file: Option<String>,

    #[clap(short, long)]
    dump: bool,
}

fn main() -> Result<()> {
    let args = Args::parse();
    // TODO: debugging, pretty-printing, better argument handling

    if args.dump {
        dump(args.file.expect("file must be provided in dump mode"))
    } else {
        repl()
    }
}
