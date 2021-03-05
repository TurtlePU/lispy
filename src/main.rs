mod ast;
mod env;
mod eval_error;
mod function;
mod parser;
mod qexpr;
mod transformers;

use std::fs::File;
use rustyline::{self, Editor};
use env::Global;
use parser::parse;
use eval_error::EvalError;

const HIST_FILE: &str = ".lisp_history";

fn main() -> rustyline::Result<()> {
    let mut editor = Editor::<()>::new();
    if let Err(_) = editor.load_history(HIST_FILE) {
        File::create(HIST_FILE)?;
    }
    let env = &mut Global::default();
    while let Ok(line) = editor.readline("lispy> ") {
        editor.add_history_entry(&line);
        match parse(&line) {
            Ok(tree) => match tree.eval(env) {
                Ok(tree) => println!("{}", tree.to_string()),
                Err(e @ EvalError::Exit) => {
                    println!("{}", e.to_string());
                    break;
                },
                Err(err) => println!("eval error: {}", err.to_string()),
            },
            Err(e) => println!("parse error: {}", e),
        }
    }
    editor.append_history(HIST_FILE)
}
