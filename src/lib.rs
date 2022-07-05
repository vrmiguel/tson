#![allow(unused_imports)]

use nom::{
    branch::alt,
    bytes::complete::{escaped, tag, take, take_while},
    character::complete::{
        alphanumeric1 as alphanumeric, char, one_of,
    },
    combinator::{cut, map, opt, rest, value},
    error::{
        context, convert_error, ContextError, ErrorKind,
        ParseError, VerboseError,
    },
    multi::{many0, separated_list0},
    number::complete::double,
    sequence::{
        delimited, preceded, separated_pair, terminated,
    },
    Err, IResult, Parser,
};

#[derive(PartialEq, Debug, Clone)]
pub enum Value<'a> {
    Float(f64),
    Boolean(bool),
    String(&'a str),
    Char(char),
    List(Vec<Value<'a>>),
    Option(Option<Box<Value<'a>>>),
}

pub fn parse_value(input: &str) -> IResult<&str, Value> {
    alt((
        parse_list.map(Value::List),
        parse_option.map(Value::Option),
        parse_double.map(Value::Float),
        parse_char.map(Value::Char),
        parse_string.map(Value::String),
        parse_boolean.map(Value::Boolean),
    ))(input)
}

fn parse_option(
    input: &str,
) -> IResult<&str, Option<Box<Value>>> {
    let parse_none = value(None, tag("None"));

    let parse_some = preceded(
        tag("Some("),
        terminated(parse_value, preceded(parse_ws, char(')'))),
    )
    .map(Box::new)
    .map(Some);

    alt((parse_none, parse_some))(input)
}

fn parse_ws(input: &str) -> IResult<&str, &str> {
    // transform is_ascii_whitespace from &self to self
    let is_ascii_whitespace =
        |ch: char| ch.is_ascii_whitespace();

    take_while(is_ascii_whitespace)(input)
}

fn parse_list(input: &str) -> IResult<&str, Vec<Value>> {
    preceded(
        char('['),
        terminated(
            separated_list0(
                preceded(parse_ws, char(',')),
                parse_value,
            ),
            preceded(parse_ws, char(']')),
        ),
    )(input)
}

fn parse_char(input: &str) -> IResult<&str, char> {
    let (rest, chr) =
        delimited(char('\''), take(1_usize), char('\''))(input)?;

    // Safety: safe unwrap since we know that there's at least
    // one element
    let chr = chr.chars().next().unwrap();

    Ok((rest, chr))
}

fn parse_string(input: &str) -> IResult<&str, &str> {
    let input = input.trim_start();
    let (rest, string) = delimited(
        char('"'),
        take_while(|ch| ch != '"'),
        char('"'),
    )(input)?;

    Ok((rest, string))
}

fn parse_boolean(input: &str) -> IResult<&str, bool> {
    let (rest, boolean) =
        alt((tag("true"), tag("false")))(input)?;

    let is_true = boolean == "true";

    Ok((rest, is_true))
}

fn parse_double(input: &str) -> IResult<&str, f64> {
    let input = input.trim_start();
    double(input)
}

#[cfg(test)]
mod tests {
    use crate::{
        parse_boolean, parse_char, parse_double, parse_list,
        parse_option, parse_string, parse_value, Value,
    };

    #[test]
    fn parses_doubles() {
        assert_eq!(parse_double(" 2.2"), Ok(("", 2.2)));
        assert_eq!(parse_double("5."), Ok(("", 5.)));
    }

    #[test]
    fn parses_booleans() {
        assert_eq!(parse_boolean("true"), Ok(("", true)));
        assert_eq!(parse_boolean("false"), Ok(("", false)));
        assert_eq!(
            parse_boolean("false false"),
            Ok((" false", false))
        );

        assert!(parse_boolean("False").is_err());
        assert!(parse_boolean("True").is_err());
        assert!(parse_boolean("1").is_err());
    }

    #[test]
    fn parses_chars() {
        assert_eq!(parse_char("'a'"), Ok(("", 'a')));
        assert_eq!(parse_char("'ã'"), Ok(("", 'ã')));
        assert!(parse_boolean("'aa'").is_err());
        assert!(parse_boolean("''").is_err());
    }

    #[test]
    fn parses_option() {
        assert_eq!(
            parse_option("Some(2)"),
            Ok(("", Some(Box::new(Value::Float(2.0)))))
        );

        assert_eq!(
            parse_option("Some('a')"),
            Ok(("", Some(Box::new(Value::Char('a')))))
        );

        assert_eq!(
            parse_option("Some(\"hey\")"),
            Ok(("", Some(Box::new(Value::String("hey")))))
        );

        assert_eq!(parse_option("None"), Ok(("", None)));
    }

    #[test]
    fn parses_strings() {
        assert_eq!(parse_string("\"hey\""), Ok(("", "hey")));
        assert_eq!(parse_string("\"2 * 2\""), Ok(("", "2 * 2")));

        assert_eq!(
            parse_string("  \"ignores leading whitespace\""),
            Ok(("", "ignores leading whitespace"))
        );
    }

    #[test]
    fn parses_values() {
        assert_eq!(
            parse_value("'B'"),
            Ok(("", Value::Char('B')))
        );
        assert_eq!(
            parse_value("\"this is a test\""),
            Ok(("", Value::String("this is a test")))
        );
    }

    #[test]
    fn parses_lists() {
        assert_eq!(parse_list("[]"), Ok(("", vec![])));

        assert_eq!(
            parse_list("['A']"),
            Ok(("", vec![Value::Char('A')]))
        );

        assert_eq!(
            parse_list("['z', 5]"),
            Ok(("", vec![Value::Char('z'), Value::Float(5.)]))
        );

        assert_eq!(
            parse_list("['f', 2.2, \"a string\"]"),
            Ok((
                "",
                vec![
                    Value::Char('f'),
                    Value::Float(2.2),
                    Value::String("a string"),
                ]
            ))
        );

        assert_eq!(
            parse_list("[[]]"),
            Ok(("", vec![Value::List(vec![])]))
        );
    }
}
