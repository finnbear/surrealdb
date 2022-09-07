use crate::ctx::Context;
use crate::err::Error;
use crate::fnc::args::Args;
use crate::sql::value::Value;
use once_cell::sync::Lazy;
use regex::Regex;
use semver::Version;
use std::char;
use uuid::Uuid;

#[rustfmt::skip] static LATITUDE_RE: Lazy<Regex> = Lazy::new(|| Regex::new("^[-+]?([1-8]?\\d(\\.\\d+)?|90(\\.0+)?)$").unwrap());
#[rustfmt::skip] static LONGITUDE_RE: Lazy<Regex> = Lazy::new(|| Regex::new("^[-+]?([1-8]?\\d(\\.\\d+)?|90(\\.0+)?)$").unwrap());

#[inline]
pub fn alphanum(_: &Context, args: Args) -> Result<Value, Error> {
	args.with_one(|arg: String| Ok(arg.chars().all(char::is_alphanumeric).into()))
		.finish("is::alphanum")
}

#[inline]
pub fn alpha(_: &Context, args: Args) -> Result<Value, Error> {
	args.with_one(|arg: String| Ok(arg.chars().all(char::is_alphabetic).into()))
		.finish("is::alpha")
}

#[inline]
pub fn ascii(_: &Context, args: Args) -> Result<Value, Error> {
	args.with_one(|arg: String| Ok(arg.is_ascii().into())).finish("is::ascii")
}

#[inline]
pub fn domain(_: &Context, args: Args) -> Result<Value, Error> {
	args.with_one(|arg: String| Ok(addr::parse_domain_name(arg.as_str()).is_ok().into()))
		.finish("is::domain")
}

#[inline]
pub fn email(_: &Context, args: Args) -> Result<Value, Error> {
	args.with_one(|arg: String| Ok(addr::parse_email_address(arg.as_str()).is_ok().into()))
		.finish("is::email")
}

#[inline]
pub fn hexadecimal(_: &Context, args: Args) -> Result<Value, Error> {
	args.with_one(|arg: String| Ok(arg.bytes().all(u8::is_ascii_hexdigit).into()))
		.finish("is::hexadecimal")
}

#[inline]
pub fn latitude(_: &Context, args: Args) -> Result<Value, Error> {
	args.with_one(|arg: String| Ok(LATITUDE_RE.is_match(arg.as_str()).into()))
		.finish("is::latitude")
}

#[inline]
pub fn longitude(_: &Context, args: Args) -> Result<Value, Error> {
	args.with_one(|arg: String| Ok(LONGITUDE_RE.is_match(arg.as_str()).into()))
		.finish("is::longitude")
}

#[inline]
pub fn numeric(_: &Context, mut args: Args) -> Result<Value, Error> {
	args.with_one(|arg: String| Ok(arg.chars().all(char::is_numeric).into()))
		.finish("is::numeric")
}

#[inline]
pub fn semver(_: &Context, mut args: Args) -> Result<Value, Error> {
	args.with_one(|arg: String| Ok(Version::parse(arg.as_str()).is_ok().into()))
		.finish("is::semver")
}

#[inline]
pub fn uuid(_: &Context, mut args: Args) -> Result<Value, Error> {
	args.with_one(|arg: String| Ok(Uuid::parse_str(arg.as_str()).is_ok().into()))
		.finish("is::uuid")
}

#[cfg(test)]
mod tests {
	use crate::sql::value::Value;

	#[test]
	fn alphanum() {
		let value = super::alphanum(&Default::default(), vec!["abc123".into()].into()).unwrap();
		assert_eq!(value, Value::True);

		let value = super::alphanum(&Default::default(), vec!["y%*".into()].into()).unwrap();
		assert_eq!(value, Value::False);
	}

	#[test]
	fn alpha() {
		let value = super::alpha(&Default::default(), vec!["abc".into()].into()).unwrap();
		assert_eq!(value, Value::True);

		let value = super::alpha(&Default::default(), vec!["1234".into()].into()).unwrap();
		assert_eq!(value, Value::False);
	}

	#[test]
	fn ascii() {
		let value = super::ascii(&Default::default(), vec!["abc".into()].into()).unwrap();
		assert_eq!(value, Value::True);

		let value = super::ascii(&Default::default(), vec!["中国".into()].into()).unwrap();
		assert_eq!(value, Value::False);
	}

	#[test]
	fn domain() {
		let value = super::domain(&Default::default(), vec!["食狮.中国".into()].into()).unwrap();
		assert_eq!(value, Value::True);

		let value = super::domain(&Default::default(), vec!["example-.com".into()].into()).unwrap();
		assert_eq!(value, Value::False);
	}

	#[test]
	fn email() {
		let input = vec!["user@[fd79:cdcb:38cc:9dd:f686:e06d:32f3:c123]".into()].into();
		let value = super::email(&Default::default(), input).unwrap();
		assert_eq!(value, Value::True);

		let input = vec!["john..doe@example.com".into()].into();
		let value = super::email(&Default::default(), input).unwrap();
		assert_eq!(value, Value::False);
	}

	#[test]
	fn hexadecimal() {
		let value = super::hexadecimal(&Default::default(), vec!["00FF00".into()].into()).unwrap();
		assert_eq!(value, Value::True);

		let value = super::hexadecimal(&Default::default(), vec!["SurrealDB".into()].into()).unwrap();
		assert_eq!(value, Value::False);
	}

	#[test]
	fn latitude() {
		let value = super::latitude(&Default::default(), vec!["-0.118092".into()].into()).unwrap();
		assert_eq!(value, Value::True);

		let value = super::latitude(&Default::default(), vec![12345.into()].into()).unwrap();
		assert_eq!(value, Value::False);
	}

	#[test]
	fn longitude() {
		let value = super::longitude(&Default::default(), vec!["51.509865".into()].into()).unwrap();
		assert_eq!(value, Value::True);

		let value = super::longitude(&Default::default(), vec![12345.into()].into()).unwrap();
		assert_eq!(value, Value::False);
	}

	#[test]
	fn numeric() {
		let value = super::numeric(&Default::default(), vec![12345.into()].into()).unwrap();
		assert_eq!(value, Value::True);

		let value = super::numeric(&Default::default(), vec!["abcde".into()].into()).unwrap();
		assert_eq!(value, Value::False);
	}

	#[test]
	fn semver() {
		let value = super::semver(&Default::default(), vec!["1.0.0".into()].into()).unwrap();
		assert_eq!(value, Value::True);

		let value = super::semver(&Default::default(), vec!["1.0".into()].into()).unwrap();
		assert_eq!(value, Value::False);
	}

	#[test]
	fn uuid() {
		let input = vec!["123e4567-e89b-12d3-a456-426614174000".into()].into();
		let value = super::uuid(&Default::default(), input).unwrap();
		assert_eq!(value, Value::True);

		let input = vec!["foo-bar".into()].into();
		let value = super::uuid(&Default::default(), input).unwrap();
		assert_eq!(value, Value::False);
	}
}
