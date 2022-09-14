//! The Rust implementation of slpy.
use slpy_rust::{repl, Result};
// use clap::Parser;

// /// The slpy programming language.
// #[derive(Parser, Debug)]
// #[clap(author, version, about, long_about = None)]
// struct Args {
//     /// Name of the person to greet
//     #[clap(short, long, value_parser)]
//     file: String,
// }

fn main() -> Result<()> {
    // let args = Args::parse();
    // TODO: debugging, pretty-printing, better argument handling

    repl()
}
