use std::{convert::TryInto, iter::IntoIterator};
use super::{ast::AST, eval_error::*};

pub fn binary_or_unary<T, U>(
    f: impl FnOnce(T, Option<T>, usize) -> EvalResult<U>
) -> impl FnOnce(Vec<T>) -> EvalResult<U> {
    move |xs| {
        let mut xs = xs.into_iter();
        let res = match (xs.next(), xs.next()) {
            (None, _) => Err(NO_ARGS),
            (Some(x), y) => f(x, y, xs.len() + 2),
        }?;
        match xs.next() {
            Some(_) => Err(EvalError::ArgsMismatch(Unexpected {
                expected: 2,
                given: xs.len() + 3,
            })),
            None => Ok(res),
        }
    }
}

pub fn unary<T, U>(f: impl FnOnce(T) -> EvalResult<U>)
    -> impl FnOnce(Vec<T>) -> EvalResult<U>
{
    binary_or_unary(move |x, y, given| match y {
        Some(_) => Err(EvalError::ArgsMismatch(given.expected(1))),
        None => f(x),
    })
}

pub fn binary<T, U>(f: impl FnOnce(T, T) -> EvalResult<U>)
    -> impl FnOnce(Vec<T>) -> EvalResult<U>
{
    binary_or_unary(move |x, y, given| match y {
        Some(y) => f(x, y),
        None => Err(EvalError::ArgsMismatch(given.expected(2))),
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
        if seq.is_empty() { return Err(NO_ARGS); }
        f(seq).map(AST::from)
    }
}
