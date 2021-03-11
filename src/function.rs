use std::vec::IntoIter;
use std::fmt;
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

impl fmt::Debug for Function {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Function::Builtin(_) => fmt.write_str("<function>"),
            Function::Lambda(f) => f.fmt(fmt),
        }
    }
}

impl PartialEq for Function {
    fn eq(&self, other: &Function) -> bool {
        match (self, other) {
            (Function::Builtin(x), Function::Builtin(y)) =>
                x as *const Builtin == y as *const Builtin,
            (Function::Lambda(x), Function::Lambda(y)) => x == y,
            _ => false,
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

#[derive(Clone, Debug, PartialEq)]
pub struct Lambda {
    context: Context,
    params: Vec<String>,
    vararg: Option<String>,
    body: QExpr,
}

impl Lambda {
    pub fn new(mut params: Vec<String>, body: QExpr) -> EvalResult {
        let tail_pos = params.iter()
            .position(|x| x == "&")
            .unwrap_or(params.len());
        let mut tail = params.drain(tail_pos..).skip(1);
        let vararg = tail.next();
        if tail.len() > 0 {
            Err(EvalError::Message("more than one param after &"))
        } else {
            std::mem::drop(tail);
            Ok(Self {
                context: Context::default(),
                params, vararg, body,
            } .ast())
        }
    }

    fn ast(self) -> AST {
        AST::Function(Function::Lambda(self))
    }

    fn call(&self, env: EnvObj, args: IntoIter<AST>) -> EvalResult {
        let (expected, given) = (self.params.len(), args.len());
        use std::cmp::Ordering::*;
        match (given.cmp(&expected), &self.vararg) {
            (Less, _) => Ok(self.curry(args).ast()),
            (Greater, None) => Err(EvalError::ArgsMismatch(Unexpected {
                expected, given
            })),
            _ => self.apply(env, args),
        }
    }

    fn curry(&self, args: IntoIter<AST>) -> Self {
        let n = args.len();
        Self {
            context: self.extend(args),
            params: Vec::from(&self.params[n..]),
            vararg: self.vararg.clone(),
            body: self.body.clone(),
        }
    }

    fn apply(&self, env: EnvObj, args: IntoIter<AST>) -> EvalResult {
        let context = match &self.vararg {
            Some(vararg) => {
                let mut args: Vec<_> = args.collect();
                let tail = args.drain(self.params.len()..).collect();
                let mut context = self.extend(args);
                context.0.insert(vararg.clone(), AST::QExpr(tail));
                context
            },
            None => self.extend(args)
        };
        self.body.clone().eval(&mut context.scope(env))
    }

    fn extend(&self, args: impl IntoIterator<Item = AST>) -> Context {
        self.context.extend(&self.params, args)
    }
}

impl ToString for Lambda {
    fn to_string(&self) -> String {
        format!("(\\ {{{}}} {})", self.params.join(" "), self.body.to_string())
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
struct Context(Bindings);

impl Context {
    fn extend(&self,
              params: &Vec<String>,
              args: impl IntoIterator<Item = AST>) -> Self {
        let mut bindings = self.0.clone();
        bindings.extend(params.iter().cloned().zip(args));
        Self(bindings)
    }

    fn scope<'a>(self, env: EnvObj<'a>) -> Scope<'a> {
        Scope::new(self.0, env)
    }
}
