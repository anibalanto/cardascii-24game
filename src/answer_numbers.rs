
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

// We parse any expr surrounded by parens, ignoring all whitespaces around those
fn parens(i: &str) -> IResult<&str, Vec<i64>> {
    delimited(space, delimited(tag("("), expr, tag(")")), space)(i)
}

fn vector(i: &str) ->  Result<Vec<i64>, &str>{
    let mut vec = Vec::<i64>::new();
    if let Ok(v) = FromStr::from_str(i) {
        vec.push(v);
    }
    Ok( vec )
}

// We transform an integer string into a i64, ignoring surrounding whitespaces
// We look for a digit suite, and try to convert it.
// If either str::from_utf8 or FromStr::from_str fail,
// we fallback to the parens parser defined above
fn factor(i: &str) -> IResult<&str, Vec<i64>> {
    alt((
        map_res(delimited(space, digit, space), vector),
        parens,
    ))(i)
}

pub fn expr(i: & str) -> IResult<&str, &mut Vec<i64>> {
    let (i, mut init) = factor(i)?;
    let mut init2 = &mut init ;

    fold_many0(
        pair(alt((char('+'), char('-'), char('/'), char('*'))), factor),
        move || init2,
        |acc: Vec<i64>, (op, val): (char, Vec<i64>)| &mut {
            for v in val.iter() {
                acc.push(*v);
            }
            acc
        },
    )(i)
}

/*#[test]
fn factor_test() {
    assert_eq!(factor("3"), Ok(("", 3)));
    assert_eq!(factor(" 12"), Ok(("", 12)));
    assert_eq!(factor("537  "), Ok(("", 537)));
    assert_eq!(factor("  24   "), Ok(("", 24)));
}

#[test]
fn expr_test() {
    assert_eq!(expr(" 1 +  2 "), Ok(("", 3)));
    assert_eq!(expr(" 12 + 6 - 4+  3"), Ok(("", 17)));
    assert_eq!(expr(" 1 + 2*3 + 4"), Ok(("", 11)));
}
*/
#[test]
fn parens_test() {
    assert_eq!(expr(" (  2 )"), Ok(("", [2])));
    vec.clear();
    assert_eq!(expr(" 2* (  3 + 4 ) "), Ok(("", [2, 3, 4])));
    vec.clear();
    assert_eq!(expr("  2*2 / ( 5 - 1) + 3"), Ok(("", [2, 2, 5, 3])));
}