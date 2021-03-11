use super::ast::AST;
use std::convert::Infallible;
use std::io;

pub type EvalResult<T = AST> = Result<T, EvalError>;

#[derive(Debug)]
pub enum EvalError {
    NotA(&'static str, AST),
    UnknownVar(String),
    ArgsMismatch(Unexpected),
    Message(&'static str),
    UserDefined(String),
    ReadError(io::Error),
    Exit,
}

pub const NO_ARGS: EvalError = EvalError::Message("no arguments");
pub const EMPTY_QEXPR: EvalError = EvalError::Message("qexpr is empty");
pub const DEF_ERROR: EvalError =
    EvalError::Message("symbol and value lists have different lengths");

impl From<Infallible> for EvalError {
    fn from(_: Infallible) -> Self {
        panic!("Like that will ever happen.")
    }
}

impl From<io::Error> for EvalError {
    fn from(err: io::Error) -> Self {
        EvalError::ReadError(err)
    }
}

impl ToString for EvalError {
    fn to_string(&self) -> String {
        use EvalError::*;
        match self {
            NotA(typ, ast) => format!("expected {}, got {}", typ, ast.typ()),
            UnknownVar(s) => format!("unknown variable: {}", s),
            ArgsMismatch(u) => format!("args mismatch: {}", u.to_string()),
            Message(s) => s.to_string(),
            UserDefined(s) => format!("exception: {}", s),
            ReadError(err) => format!("error reading file: {}", err),
            Exit => "exiting.".to_string(),
        }
    }
}

#[derive(Debug)]
pub struct Unexpected {
    pub expected: usize,
    pub given: usize,
}

impl ToString for Unexpected {
    fn to_string(&self) -> String {
        format!("{} expected, {} given", self.expected, self.given)
    }
}

pub trait Expected {
    fn expected(self, expected: usize) -> EvalError;
}

impl Expected for usize {
    fn expected(self, expected: usize) -> EvalError {
        EvalError::ArgsMismatch(Unexpected { expected, given: self })
    }
}
