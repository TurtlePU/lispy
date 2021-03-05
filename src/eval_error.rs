use super::ast::AST;

pub type EvalResult<T = AST> = Result<T, EvalError>;

pub enum EvalError {
    NotA(&'static str, AST),
    UnknownVar(String),
    ArgsMismatch(Unexpected),
    Message(&'static str),
    Exit,
}

pub const NO_ARGS: EvalError = EvalError::Message("no arguments");
pub const EMPTY_QEXPR: EvalError = EvalError::Message("qexpr is empty");
pub const DEF_ERROR: EvalError =
    EvalError::Message("symbol and value lists have different lengths");

impl ToString for EvalError {
    fn to_string(&self) -> String {
        use EvalError::*;
        match self {
            NotA(typ, ast) => format!("expected {}, got {}", typ, ast.typ()),
            UnknownVar(s) => format!("unknown variable: {}", s),
            ArgsMismatch(u) => format!("args mismatch: {}", u.to_string()),
            Message(s) => s.to_string(),
            Exit => "exiting.".to_string(),
        }
    }
}

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
    fn expected(self, expected: usize) -> Unexpected;
}

impl Expected for usize {
    fn expected(self, expected: usize) -> Unexpected {
        Unexpected { expected, given: self }
    }
}
