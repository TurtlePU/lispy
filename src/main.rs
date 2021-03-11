mod ast;
mod env;
mod eval_error;
mod function;
mod parser;
mod qexpr;
mod transformers;

use std::fs::File;
use rustyline::{self, Editor};
use env::{Global, load};
use parser::parse;
use eval_error::EvalError;

const HIST_FILE: &str = ".lisp_history";

fn main() {
    let mut args = std::env::args();
    args.next().unwrap();
    if args.len() > 0 {
        run_interpreter(args);
    } else if let Err(err) = run_repl() {
        println!("{}", err);
    }
}

fn run_interpreter(args: std::env::Args) {
    let env = &mut Global::default();
    for filename in args {
        if let Err(err) = load(env, filename) {
            println!("load error: {}", err.to_string());
        }
    }
}

fn run_repl() -> rustyline::Result<()> {
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
