use std::collections::HashMap;
use std::path::Path;
use std::fs;
use super::parser::parse;
use super::{ast::*, function::*, eval_error::*};

pub type EnvObj<'a> = &'a mut dyn Env;

pub trait Env {
    fn get(&self, key: String) -> EvalResult<AST>;
    fn define(&mut self, bindings: BindingsVec);
    fn assign(&mut self, bindings: BindingsVec);
}

pub fn load<P>(env: EnvObj, file: P) -> EvalResult where P: AsRef<Path> {
    let content = fs::read_to_string(file)?;
    for line in content.lines() {
        match parse(line) {
            Ok(expr) => if let Err(err) = expr.eval(env) {
                println!("{}", err.to_string());
            },
            Err(err) => println!("{}", err),
        }
    }
    Ok(AST::default())
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
            ("<", less),
            ("==", eq),
            ("list", list),
            ("head", head),
            ("tail", tail),
            ("join", join),
            ("eval", eval),
            ("def", def),
            ("if", iff),
            ("load", load),
            ("print", print),
            ("error", error),
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
        binary_or_unary(|x: i128, y| Ok(match y {
            Some(y) => x - y,
            None => -x,
        }))(args)
    }

    pub fn mul(_: EnvObj, args: Vec<AST>) -> EvalResult {
        oftype(|xs: Vec<i128>| Ok(xs.into_iter().product()))(args)
    }

    pub fn div(_: EnvObj, args: Vec<AST>) -> EvalResult {
        binary(|x: i128, y| match x.checked_div(y) {
            Some(z) => Ok(z),
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
        unary(QExpr::head)(args)
    }

    pub fn tail(_: EnvObj, args: Vec<AST>) -> EvalResult {
        unary(QExpr::tail)(args)
    }

    pub fn join(_: EnvObj, args: Vec<AST>) -> EvalResult {
        oftype(|xs: Vec<QExpr>| Ok(QExpr::from(xs.concat())))(args)
    }

    pub fn eval(env: EnvObj, args: Vec<AST>) -> EvalResult {
        unary(|x: QExpr| x.eval(env))(args)
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
        binary(|defs: QExpr, body| Lambda::new(defs.symbols()?, body))(args)
    }

    pub fn exit(_: EnvObj, _: Vec<AST>) -> EvalResult {
        Err(EvalError::Exit)
    }

    pub fn less(_: EnvObj, args: Vec<AST>) -> EvalResult {
        binary(|x: i128, y| Ok(bool_int(x < y)))(args)
    }

    pub fn eq(_: EnvObj, args: Vec<AST>) -> EvalResult {
        binary(|x: AST, y| Ok(AST::Number(bool_int(x == y))))(args)
    }

    pub fn iff(env: EnvObj, args: Vec<AST>) -> EvalResult {
        let mut args = args.into_iter();
        let num = args.next().ok_or(NO_ARGS)?.number()?;
        let left = args.next().ok_or(1.expected(3))?.qexpr()?;
        let right = args.next().ok_or(2.expected(3))?.qexpr()?;
        if num == 0 {
            right.eval(env)
        } else {
            left.eval(env)
        }
    }

    pub fn load(env: EnvObj, args: Vec<AST>) -> EvalResult {
        unary(|file: AST| super::load(env, file.literal()?))(args)
    }

    pub fn print(_: EnvObj, args: Vec<AST>) -> EvalResult {
        let joined = args.iter()
            .map(AST::to_string).collect::<Vec<_>>().join(" ");
        println!("{}", joined);
        Ok(AST::default())
    }

    pub fn error(_: EnvObj, args: Vec<AST>) -> EvalResult {
        unary(|err: AST| -> EvalResult {
            Err(EvalError::UserDefined(err.literal()?))
        })(args)
    }

    fn bool_int(x: bool) -> i128 {
        if x { 1 } else { 0 }
    }
}
