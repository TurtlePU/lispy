use std::convert::TryInto;
use super::{
    env::EnvObj,
    eval_error::{EvalError, EvalResult},
    function::Function,
    qexpr::{QExpr, pprint},
    transformers::sequence,
};

#[derive(Clone, Debug, PartialEq)]
pub enum AST {
    Number(i128),
    Symbol(String),
    Literal(String),
    SExpr(Vec<AST>),
    QExpr(QExpr),
    Function(Function),
}

impl Default for AST {
    fn default() -> Self {
        AST::SExpr(vec![])
    }
}

impl From<i128> for AST {
    fn from(x: i128) -> Self {
        AST::Number(x)
    }
}

impl From<QExpr> for AST {
    fn from(x: QExpr) -> Self {
        AST::QExpr(x)
    }
}

impl TryInto<i128> for AST {
    type Error = EvalError;

    fn try_into(self) -> Result<i128, Self::Error> {
        self.number()
    }
}

impl TryInto<QExpr> for AST {
    type Error = EvalError;

    fn try_into(self) -> Result<QExpr, Self::Error> {
        self.qexpr()
    }
}

impl AST {
    pub fn eval(self, env: EnvObj) -> EvalResult {
        match self {
            AST::Symbol(var) => env.get(var),
            AST::SExpr(mut expr) if expr.len() == 1 =>
                expr.pop().unwrap().eval(env),
            AST::SExpr(exprs) if !exprs.is_empty() => {
                let mut exprs = sequence(exprs, |x| x.eval(env))?.into_iter();
                let fun = exprs.next().unwrap().function()?;
                fun.call(env, exprs)
            },
            ast => Ok(ast),
        }
    }

    pub fn typ(&self) -> &'static str {
        match self {
            AST::Number(_) => "number",
            AST::Symbol(_) => "symbol",
            AST::Literal(_) => "string",
            AST::SExpr(_) => "S-expr",
            AST::QExpr(_) => "Q-expr",
            AST::Function(_) => "function",
        }
    }

    pub fn number(self) -> EvalResult<i128> {
        match self {
            AST::Number(x) => Ok(x),
            ast => Err(EvalError::NotA("number", ast)),
        }
    }

    pub fn qexpr(self) -> EvalResult<QExpr> {
        match self {
            AST::QExpr(xs) => Ok(xs),
            ast => Err(EvalError::NotA("Q-expr", ast)),
        }
    }

    pub fn symbol(self) -> EvalResult<String> {
        match self {
            AST::Symbol(sym) => Ok(sym),
            ast => Err(EvalError::NotA("symbol", ast)),
        }
    }

    pub fn literal(self) -> EvalResult<String> {
        match self {
            AST::Literal(lit) => Ok(lit),
            ast => Err(EvalError::NotA("string", ast)),
        }
    }

    fn function(self) -> EvalResult<Function> {
        match self {
            AST::Function(fun) => Ok(fun),
            ast => Err(EvalError::NotA("function", ast)),
        }
    }
}

impl ToString for AST {
    fn to_string(&self) -> String {
        match self {
            AST::Number(num) => num.to_string(),
            AST::Symbol(sym) => sym.clone(),
            AST::Literal(string) => format!("\"{}\"", escaped(string)),
            AST::SExpr(asts) => pprint("(", asts, ")"),
            AST::QExpr(asts) => asts.to_string(),
            AST::Function(fun) => fun.to_string(),
        }
    }
}

fn escaped(string: &String) -> String {
    string
        .replace("\n", "\\n")
        .replace("\t", "\\t")
        .replace("\r", "\\r")
        .replace("\"", "\\\"")
}
