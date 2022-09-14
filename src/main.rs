//! The Rust implementation of slpy.
use slpy_rust::{repl, Result, dump};
use clap::Parser;

/// The slpy programming language.
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// Name of the person to greet
    #[clap(short, long)]
    file: String,

    #[clap(short, long)]
    dump: bool
}

fn main() -> Result<()> {
    let args = Args::parse();
    // TODO: debugging, pretty-printing, better argument handling

    if args.dump {
        dump(args.file)
    } else {
        repl()
    }
}
