//! The REPL.
use std::io::Write;

use crate::ast::{Ast, Context, Expn, Stmt};
use crate::tokenizer::Tokenizer;
use crate::Result;

/// Run the REPL.
///
/// # Errors
/// If the code is wrong.
pub fn repl() -> Result<()> {
    let mut source = String::new();
    let mut ctx = Context::default();
    loop {
        // TODO: handle EOF
        print!(">>> ");
        std::io::stdout().flush().expect("can flush stdout");
        if std::io::stdin().read_line(&mut source).is_ok() {
            let tokens = Tokenizer::lex(source.as_str())?;
            if let Ok(n) = Expn::parse_and_eval(tokens, &mut ctx) {
                println!("{}", n);
            } else {
                let tokens = Tokenizer::lex(source.as_str())?;
                Stmt::parse_and_eval(tokens, &mut ctx)?;
            }
        }
        source.clear();
    }
}
