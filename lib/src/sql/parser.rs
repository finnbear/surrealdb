use crate::err::Error;
use crate::sql::error::Error::{RecursionLimit, ParserError};
use crate::sql::query::{query, Query};
use crate::sql::thing::Thing;
use crate::sql::value::Value;
use nom::Err;
use std::str;
use crate::sql::error::IResult;

pub struct ParseDepth(u8);

impl ParseDepth {
	pub fn top() -> Self {
		Self(0)
	}

	pub fn dive(self) -> Result<Self, Err<crate::sql::Error<&'static str>>> {
		if self.0 < 24 {
			Ok(Self(self.0 + 1))
		} else {
			Err(Err::Error(RecursionLimit))
		}
	}
}

pub fn parse(input: &str) -> Result<Query, Error> {
	parse_impl(input, query)
}

pub fn thing(input: &str) -> Result<Thing, Error> {
	parse_impl(input, super::thing::thing)
}

pub fn json(input: &str) -> Result<Value, Error> {
	parse_impl(input, super::value::json)
}

fn parse_impl<O>(input: &str, parser: impl Fn(&str, ParseDepth) -> IResult<&str, O>) -> Result<O, Error> {
	match input.trim().len() {
		0 => Err(Error::QueryEmpty),
		_ => match parser(input, ParseDepth::top()) {
			Ok((_, query)) => Ok(query),
			Err(Err::Error(e)) | Err(Err::Failure(e)) => Err(match e {
				ParserError(e) => {
					let (s, l, c) = locate(input, e);
					Error::InvalidQuery {
						line: l,
						char: c,
						sql: s.to_string(),
					}
				},
				// TODO: Replace with ComputationDepthExceeded when
				// https://github.com/surrealdb/surrealdb/pull/241 is merged.
				RecursionLimit => Error::TooManySubqueries
			}),
			_ => unreachable!(),
		},
	}
}

fn truncate(s: &str, l: usize) -> &str {
	match s.char_indices().nth(l) {
		None => s,
		Some((i, _)) => &s[..i],
	}
}

fn locate<'a>(input: &str, tried: &'a str) -> (&'a str, usize, usize) {
	let index = input.len() - tried.len();
	let tried = truncate(tried, 100);
	let lines = input.split('\n').collect::<Vec<&str>>();
	let lines = lines.iter().map(|l| l.len()).enumerate();
	let (mut total, mut chars) = (0, 0);
	for (line, size) in lines {
		total += size + 1;
		if index < total {
			let line_num = line + 1;
			let char_num = index - chars;
			return (tried, line_num, char_num);
		}
		chars += size + 1;
	}
	(tried, 0, 0)
}

#[cfg(test)]
mod tests {

	use super::*;

	#[test]
	fn no_ending() {
		let sql = "SELECT * FROM test";
		let res = parse(sql);
		assert!(res.is_ok());
	}

	#[test]
	fn parse_query_string() {
		let sql = "SELECT * FROM test;";
		let res = parse(sql);
		assert!(res.is_ok());
	}

	#[test]
	fn trim_query_string() {
		let sql = "    SELECT    *    FROM    test    ;    ";
		let res = parse(sql);
		assert!(res.is_ok());
	}

	#[test]
	fn parse_complex_rubbish() {
		let sql = "    SELECT    *    FROM    test    ; /* shouldbespace */ ;;;    ";
		let res = parse(sql);
		assert!(res.is_ok());
	}

	#[test]
	fn parse_complex_failure() {
		let sql = "    SELECT    *    FROM    { }} ";
		let res = parse(sql);
		assert!(res.is_err());
	}

	#[test]
	fn parser_try() {
		let sql = "
			SELECT
				*,
				tags[$].value,
				3s as duration,
				1.345 AS number,
				test AS `some thing`,
				'2012-04-23T18:25:43.511Z' AS utctime,
				'2012-04-23T18:25:43.511-08:00' AS pacifictime,
				{ key: (3 + 1 + 2), other: 9 * 7, 'some thing': { otherkey: 'text', } } AS object
			FROM $param, test, temp, test:thingy, |test:10|, |test:1..10|
			WHERE IF true THEN 'YAY' ELSE 'OOPS' END
				AND (0.1341, 0.5719) INSIDE { type: 'Polygon', coordinates: [[[0.1341, 0.5719], [0.1341, 0.5719]]] }
				AND (3 + 3 * 4)=6
				AND 3 + 3 * 4 = 6
				AND ages CONTAINS 18
				AND if IS true
			SPLIT test.things
			VERSION '2019-01-01'
			TIMEOUT 2w;

			CREATE person SET name = 'Tobie', age += 18;
		";
		let res = parse(sql);
		assert!(res.is_ok());
		let tmp = res.unwrap();

		let enc: Vec<u8> = Vec::from(&tmp);
		let dec: Query = Query::from(enc);
		assert_eq!(tmp, dec);
	}
}
