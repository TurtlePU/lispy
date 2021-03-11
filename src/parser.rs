use std::str::FromStr;
use super::{ast::AST, qexpr::QExpr};
use nom::{
    IResult,
    branch::alt,
    bytes::complete::tag,
    combinator::{all_consuming, into, map, map_res, opt, recognize, value},
    character::complete::*,
    error::Error,
    multi::{many0, many1},
    sequence::{delimited, preceded},
};

pub type MyResult<'a, T = AST, E = Error<&'a str>> = IResult<&'a str, T, E>;

pub fn parse(string: &str) -> Result<AST, nom::Err<Error<&str>>> {
    let (_, result) = all_consuming(map(parse_many, AST::SExpr))(string)?;
    Ok(result)
}

fn parse_many(string: &str) -> MyResult<Vec<AST>> {
    many0(parse_ast)(string)
}

fn parse_ast(string: &str) -> MyResult {
    delimited(
        spaces,
        alt((
            map(parse_number, AST::Number),
            map(parse_symbol, AST::Symbol),
            map(parse_string, AST::Literal),
            map(parse_sexpr, AST::SExpr),
            map(parse_qexpr, AST::QExpr),
        )),
        spaces
    )(string)
}

fn parse_number(string: &str) -> MyResult<i128> {
    let pattern = preceded(opt(tag("-")), digit1);
    map_res(recognize(pattern), FromStr::from_str)(string)
}

fn parse_symbol(string: &str) -> MyResult<String> {
    let pattern = many1(alt((
        value((), alphanumeric1),
        value((), one_of("_+-*/\\=<>!&|")),
    )));
    map(recognize(pattern), String::from)(string)
}

fn parse_string(string: &str) -> MyResult<String> {
    let pattern = many1(alt((
        value('\n', tag("\\n")),
        value('\r', tag("\\r")),
        value('\t', tag("\\r")),
        preceded(tag("\\"), anychar),
        none_of("\""),
    )));
    delimited(tag("\""), map(pattern, |x| x.into_iter().collect()), tag("\""))
        (string)
}

fn parse_sexpr(string: &str) -> MyResult<Vec<AST>> {
    delimited(tag("("), parse_many, tag(")"))(string)
}

fn parse_qexpr(string: &str) -> MyResult<QExpr> {
    into(delimited(tag("{"), parse_many, tag("}")))(string)
}

fn spaces(string: &str) -> MyResult<Option<&str>> {
    delimited(multispace0, opt(comments), multispace0)(string)
}

fn comments(string: &str) -> MyResult<&str> {
    preceded(tag(";"), not_line_ending)(string)
}
