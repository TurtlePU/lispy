use std::collections::HashMap;
use super::{ast::*, function::*, eval_error::*};

pub type EnvObj<'a> = &'a mut dyn Env;

pub trait Env {
    fn get(&self, key: String) -> EvalResult<AST>;
    fn define(&mut self, bindings: BindingsVec);
    fn assign(&mut self, bindings: BindingsVec);
}

pub type Bindings = HashMap<String, AST>;
type BindingsVec = Vec<(String, AST)>;

pub struct Global {
    bindings: Bindings,
}

impl Default for Global {
    fn default() -> Self {
        use builtins::*;
        let bindings: Vec<(&str, Builtin)> = vec![
            ("+", add),
            ("-", sub),
            ("*", mul),
            ("/", div),
            ("\\", lambda),
            ("=", assign),
            ("list", list),
            ("head", head),
            ("tail", tail),
            ("join", join),
            ("eval", eval),
            ("def", def),
            ("exit", exit),
        ];
        let bindings = bindings.into_iter().map(|(s, f)| {
            (s.to_string(), AST::Function(Function::Builtin(f)))
        }).collect();
        Self { bindings }
    }
}

impl Env for Global {
    fn get(&self, key: String) -> EvalResult<AST> {
        match self.bindings.get(&key) {
            Some(value) => Ok(value.clone()),
            None => Err(EvalError::UnknownVar(key)),
        }
    }

    fn define(&mut self, bindings: BindingsVec) {
        self.bindings.extend(bindings);
    }

    fn assign(&mut self, bindings: BindingsVec) {
        self.define(bindings);
    }
}

pub struct Scope<'a> {
    bindings: Bindings,
    parent: EnvObj<'a>,
}

impl<'a> Scope<'a> {
    pub fn new(bindings: Bindings, parent: EnvObj<'a>) -> Self {
        Self { bindings, parent }
    }
}

impl<'a> Env for Scope<'a> {
    fn get(&self, key: String) -> EvalResult<AST> {
        match self.bindings.get(&key) {
            Some(value) => Ok(value.clone()),
            None => self.parent.get(key),
        }
    }

    fn define(&mut self, bindings: BindingsVec) {
        self.parent.define(bindings);
    }

    fn assign(&mut self, bindings: BindingsVec) {
        self.bindings.extend(bindings);
    }
}

mod builtins {
    use crate::{
        ast::AST,
        eval_error::*,
        function::*,
        transformers::*,
        qexpr::QExpr,
    };
    use super::{EnvObj, BindingsVec};

    pub fn add(_: EnvObj, args: Vec<AST>) -> EvalResult {
        oftype(|xs: Vec<i128>| Ok(xs.into_iter().sum()))(args)
    }

    pub fn sub(_: EnvObj, args: Vec<AST>) -> EvalResult {
        binary_or_unary(|x: AST, y, _| Ok(AST::Number(match y {
            Some(y) => x.number()? - y.number()?,
            None => -x.number()?,
        })))(args)
    }

    pub fn mul(_: EnvObj, args: Vec<AST>) -> EvalResult {
        oftype(|xs: Vec<i128>| Ok(xs.into_iter().product()))(args)
    }

    pub fn div(_: EnvObj, args: Vec<AST>) -> EvalResult {
        binary(|x: AST, y| match x.number()?.checked_div(y.number()?) {
            Some(z) => Ok(AST::Number(z)),
            None => Err(EvalError::Message("div by zero")),
        })(args)
    }

    pub fn list(_: EnvObj, args: Vec<AST>) -> EvalResult {
        match args.is_empty() {
            true => Err(NO_ARGS),
            false => Ok(AST::QExpr(QExpr::from(args))),
        }
    }

    pub fn head(_: EnvObj, args: Vec<AST>) -> EvalResult {
        unary(|x: AST| x.qexpr()?.head().map(AST::QExpr))(args)
    }

    pub fn tail(_: EnvObj, args: Vec<AST>) -> EvalResult {
        unary(|x: AST| x.qexpr()?.tail().map(AST::QExpr))(args)
    }

    pub fn join(_: EnvObj, args: Vec<AST>) -> EvalResult {
        Ok(AST::QExpr(QExpr::from(sequence(args, AST::qexpr)?.concat())))
    }

    pub fn eval(env: EnvObj, args: Vec<AST>) -> EvalResult {
        unary(|x: AST| x.qexpr()?.eval(env))(args)
    }

    pub fn def(env: EnvObj, args: Vec<AST>) -> EvalResult {
        env.define(bindings(args)?);
        Ok(AST::default())
    }

    pub fn assign(env: EnvObj, args: Vec<AST>) -> EvalResult {
        env.assign(bindings(args)?);
        Ok(AST::default())
    }

    fn bindings(args: Vec<AST>) -> EvalResult<BindingsVec> {
        let mut args = args.into_iter();
        match args.next() {
            Some(syms) => {
                let syms = syms.qexpr()?.symbols()?.into_iter();
                if syms.len() != args.len() {
                    return Err(DEF_ERROR);
                }
                Ok(syms.zip(args).collect())
            },
            None => Err(NO_ARGS),
        }
    }

    pub fn lambda(_: EnvObj, args: Vec<AST>) -> EvalResult {
        binary(|defs: AST, body| {
            Ok(Lambda::new(defs.qexpr()?.symbols()?, body.qexpr()?))
        })(args)
    }

    pub fn exit(_: EnvObj, _: Vec<AST>) -> EvalResult {
        Err(EvalError::Exit)
    }
}
