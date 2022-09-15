//! The Rust implementation of slpy.
use clap::Parser;
use slpy_rust::{dump, repl, run};

/// The slpy programming language.
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// Name of the person to greet
    #[clap(value_parser)]
    file: Option<String>,

    #[clap(short, long)]
    dump: bool,
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    // TODO: debugging, pretty-printing, better argument handling

    if let Some(file) = args.file {
        if args.dump {
            dump(file)?;
        } else {
            run(file)?;
        }
    } else {
        repl()?;
    }

    Ok(())
}
