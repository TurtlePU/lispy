use std::borrow::Borrow;
use super::{ast::AST, env::EnvObj, eval_error::*, transformers::sequence};

#[derive(Clone)]
pub struct QExpr(Vec<AST>);

impl From<Vec<AST>> for QExpr {
    fn from(vec: Vec<AST>) -> Self {
        Self(vec)
    }
}

impl QExpr {
    pub fn head(self) -> EvalResult<QExpr> {
        match self.0.into_iter().next() {
            Some(head) => Ok(QExpr(vec![head])),
            None => Err(EMPTY_QEXPR),
        }
    }

    pub fn tail(mut self) -> EvalResult<QExpr> {
        if self.0.is_empty() {
            return Err(EMPTY_QEXPR);
        }
        self.0.reverse();
        self.0.pop();
        self.0.reverse();
        Ok(self)
    }

    pub fn symbols(self) -> EvalResult<Vec<String>> {
        sequence(self.0.into_iter(), AST::symbol)
    }

    pub fn eval(self, env: EnvObj) -> EvalResult {
        AST::SExpr(self.0).eval(env)
    }
}

impl Borrow<[AST]> for QExpr {
    fn borrow(&self) -> &[AST] {
        self.0.borrow()
    }
}

impl ToString for QExpr {
    fn to_string(&self) -> String {
        pprint("{", &self.0, "}")
    }
}

pub fn pprint(before: &str, asts: &Vec<AST>, after: &str) -> String {
    let strings = asts.iter().map(AST::to_string).collect::<Vec<_>>();
    before.to_owned() + &strings.join(" ") + after
}
