use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::char,
    character::complete::{digit1 as digit, space0 as space},
    combinator::map_res,
    multi::fold_many0,
    sequence::{delimited, pair},
    IResult,
};

// Parser definition

use std::str::FromStr;

pub fn analize(i: &str) -> IResult<&str, i64> {
    let mut vec = Vec::<i64>::new();

    let mut expr:fn(i: &str) -> IResult<&str, i64> = |i| {Ok(("", 3 as i64))};
    // We parse any expr surrounded by parens, ignoring all whitespaces around those
    let parens = |i| {
        delimited(space, delimited(tag("("), expr, tag(")")), space)(i)
    };


    // We transform an integer string into a i64, ignoring surrounding whitespaces
    // We look for a digit suite, and try to convert it.
    // If either str::from_utf8 or FromStr::from_str fail,
    // we fallback to the parens parser defined above
    let factor = |i| {
        alt((
            map_res(delimited(space, digit, space), FromStr::from_str),
            parens,
        ))(i)
    };


    // We read an initial factor and for each time we find
    // a * or / operator followed by another factor, we do
    // the math by folding everything
    let term = |i| {
        let (i, init) = factor(i)?;

        fold_many0(
            pair(alt((char('*'), char('/'))), factor),
            move || init,
            |acc, (op, val): (char, i64)| {
                if op == '*' {
                    acc * val
                } else {
                    acc / val
                }
            },
        )(i)
    };


    let expr2 = & |i| {
        let (i, init) = term(i)?;

        fold_many0(
            pair(alt((char('+'), char('-'))), term),
            move || init,
            |acc, (op, val): (char, i64)| {
                if op == '+' {
                    acc + val
                } else {
                    acc - val
                }
            },
        )(i)
    };

    expr = *expr2;

    expr(i)

}

/*#[test]
fn factor_test() {
    assert_eq!(factor("3"), Ok(("", 3)));
    assert_eq!(factor(" 12"), Ok(("", 12)));
    assert_eq!(factor("537  "), Ok(("", 537)));
    assert_eq!(factor("  24   "), Ok(("", 24)));
}

#[test]
fn term_test() {
    assert_eq!(term(" 12 *2 /  3"), Ok(("", 8)));
    assert_eq!(term(" 2* 3  *2 *2 /  3"), Ok(("", 8)));
    assert_eq!(term(" 48 /  3/2"), Ok(("", 8)));
}

#[test]
fn expr_test() {
    assert_eq!(expr(" 1 +  2 "), Ok(("", 3)));
    assert_eq!(expr(" 12 + 6 - 4+  3"), Ok(("", 17)));
    assert_eq!(expr(" 1 + 2*3 + 4"), Ok(("", 11)));
}*/

#[test]
fn parens_test() {
    assert_eq!(analize(" (  2 )"), Ok(("", 2)));
    assert_eq!(analize(" 2* (  3 + 4 ) "), Ok(("", 14)));
    assert_eq!(analize("  2*2 / ( 5 - 1) + 3"), Ok(("", 4)));
}