use std::{convert::TryInto, iter::IntoIterator};
use super::{ast::AST, eval_error::*};

pub fn binary_or_unary<T, U, E>(f: impl FnOnce(T, Option<T>) -> EvalResult<U>)
    -> impl FnOnce(Vec<AST>) -> EvalResult
where AST: From<U> + TryInto<T, Error = E>,
      EvalError: From<E>,
{
    move |xs| {
        let mut xs = xs.into_iter();
        let res = match (xs.next(), xs.next()) {
            (None, _) => Err(NO_ARGS),
            (Some(x), y) => f(x.try_into()?, y.map(AST::try_into).transpose()?),
        }?;
        match xs.next() {
            Some(_) => Err((xs.len() + 3).expected(2)),
            None => Ok(res.into()),
        }
    }
}

pub fn unary<T, U, E>(f: impl FnOnce(T) -> EvalResult<U>)
    -> impl FnOnce(Vec<AST>) -> EvalResult
where AST: From<U> + TryInto<T, Error = E>,
      EvalError: From<E>,
{
    move |mut xs| match xs.len() {
        0 => Err(NO_ARGS),
        1 => f(xs.pop().unwrap().try_into()?).map(AST::from),
        given => Err(given.expected(1)),
    }
}

pub fn binary<T, U, E>(f: impl FnOnce(T, T) -> EvalResult<U>)
    -> impl FnOnce(Vec<AST>) -> EvalResult
where AST: From<U> + TryInto<T, Error = E>,
      EvalError: From<E>,
{
    binary_or_unary(move |x, y| match y {
        Some(y) => f(x, y),
        None => Err(1.expected(2)),
    })
}

pub fn sequence<T, U, E>(
    iter: impl IntoIterator<Item = T>,
    mapper: impl FnMut(T) -> Result<U, E>,
) -> Result<Vec<U>, E> {
    iter.into_iter().map(mapper).collect::<Result<_, _>>()
}

pub fn oftype<T, I>(f: impl FnOnce(Vec<T>) -> EvalResult<T>)
    -> impl FnOnce(I) -> EvalResult
where I: IntoIterator<Item = AST>,
      AST: TryInto<T, Error = EvalError> + From<T>,
{
    |xs| {
        let seq = sequence(xs, AST::try_into)?;
        if seq.is_empty() {
            Err(NO_ARGS)
        } else {
            f(seq).map(AST::from)
        }
    }
}
