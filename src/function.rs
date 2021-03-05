use std::vec::IntoIter;
use super::{env::*, ast::AST, eval_error::*, qexpr::QExpr};

pub type Builtin = fn(EnvObj, Vec<AST>) -> EvalResult;

#[derive(Clone)]
pub enum Function {
    Builtin(Builtin),
    Lambda(Lambda),
}

impl Function {
    pub fn call(&self, env: EnvObj, args: IntoIter<AST>) -> EvalResult {
        match self {
            Function::Builtin(f) => f(env, args.collect()),
            Function::Lambda(f) => f.call(env, args),
        }
    }
}

impl ToString for Function {
    fn to_string(&self) -> String {
        match self {
            Function::Builtin(_) => "<function>".to_string(),
            Function::Lambda(f) => f.to_string(),
        }
    }
}

#[derive(Clone)]
pub struct Lambda {
    context: Context,
    params: Vec<String>,
    body: QExpr,
}

impl Lambda {
    pub fn new(params: Vec<String>, body: QExpr) -> AST {
        Self { context: Context::default(), params, body } .ast()
    }

    fn ast(self) -> AST {
        AST::Function(Function::Lambda(self))
    }

    fn call(&self, env: EnvObj, args: IntoIter<AST>) -> EvalResult {
        let (expected, given) = (self.params.len(), args.len());
        use std::cmp::Ordering::*;
        match given.cmp(&expected) {
            Less => Ok(self.curry(args).ast()),
            Equal => self.apply(env, args),
            Greater => Err(EvalError::ArgsMismatch(
                Unexpected { expected, given }
            )),
        }
    }

    fn curry(&self, args: IntoIter<AST>) -> Self {
        let n = args.len();
        Self {
            context: self.extend(args),
            params: Vec::from(&self.params[n..]),
            body: self.body.clone(),
        }
    }

    fn apply(&self, env: EnvObj, args: IntoIter<AST>) -> EvalResult {
        self.body.clone().eval(&mut self.extend(args).scope(env))
    }

    fn extend(&self, args: IntoIter<AST>) -> Context {
        self.context.extend(&self.params, args)
    }
}

impl ToString for Lambda {
    fn to_string(&self) -> String {
        format!("(\\ {{{}}} {})", self.params.join(" "), self.body.to_string())
    }
}

#[derive(Clone, Default)]
struct Context(Bindings);

impl Context {
    fn extend(&self, params: &Vec<String>, args: IntoIter<AST>) -> Self {
        let mut bindings = self.0.clone();
        bindings.extend(params.iter().cloned().zip(args));
        Self(bindings)
    }

    fn scope<'a>(self, env: EnvObj<'a>) -> Scope<'a> {
        Scope::new(self.0, env)
    }
}
