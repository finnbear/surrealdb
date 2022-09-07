use crate::ctx::Context;
use crate::err::Error;
use crate::sql::value::Value;
use std::ops::Deref;

pub type Args = ArgsImpl<0, 0, 0, 0>;

/// FIXME: Use bool instead of u8 once const generic ADTs are stable.
#[repr(transparent)]
pub enum ArgsImpl<const NONE: u8, const ONE: u8, const TWO: u8, const THREE: u8> {
	Pending(Vec<Value>),
	Done(Result<Value, Error>),
}

impl From<Vec<Value>> for ArgsImpl<0, 0, 0, 0> {
	fn from(args: Vec<Value>) -> Self {
		Self::Pending(args)
	}
}

impl ArgsImpl<0, 0, 0, 0> {
	pub fn into_inner(self) -> Vec<Value> {
		self.0
	}
}

impl<const ONE: u8, const TWO: u8, const THREE: u8> ArgsImpl<0, ONE, TWO, THREE> {
	pub fn with_none(self, f: impl FnOnce() -> Result<Value, Error>) -> Self<1, ONE, TWO, THREE> {
		match self {
			Self::Pending(args) if args.is_empty() => Self::Done(f()),
			r => r,
		}
	}
}

impl<const NONE: u8, const TWO: u8, const THREE: u8> ArgsImpl<NONE, 0, TWO, THREE> {
	pub fn one<A: ArgFrom>(
		&mut self
	) -> Option<A> {
		let mut ret = None;
		*self = self.with_one(|value| {
			ret = Some(value);
			Ok(Value::None)
		});
		ret
	}

	pub fn with_one<A: ArgFrom>(
		self,
		f: impl FnOnce(A) -> Result<Value, Error>,
	) -> Self<NONE, 1, TWO, THREE> {
		match self {
			Self::Pending(args) => match args.try_into() {
				Ok([value]) => Self::Done(f(A::arg_from(value))),
				Err(args) => Self::Pending(args),
			},
			r => r,
		}
	}
}

impl<const NONE: u8, const ONE: u8, const THREE: u8> ArgsImpl<NONE, ONE, 0, THREE> {
	pub fn two<A1: ArgFrom, A2: ArgFrom>(
		&mut self
	) -> Option<(A1, A2)> {
		let mut ret = None;
		*self = self.with_two(|value1, value2| {
			ret = Some((value1, value2));
			Ok(Value::None)
		});
		ret
	}

	pub fn with_two<A1: ArgFrom, A2: ArgFrom>(
		self,
		f: impl FnOnce(A1, A2) -> Result<Value, Error>,
	) -> Self<NONE, ONE, 1, THREE> {
		match self {
			Self::Pending(args) => match args.try_into() {
				Ok([value1, value2]) => Self::Done(f(A1::arg_from(value1), A2::arg_from(value2))),
				Err(args) => Self::Pending(args),
			},
			r => r,
		}
	}
}

impl<const NONE: u8, const ONE: u8, const TWO: u8> ArgsImpl<NONE, ONE, TWO, 0> {
	pub fn with_three(
		self,
		f: impl FnOnce(Value, Value) -> Result<Value, Error>,
	) -> Self<NONE, ONE, TWO, 1> {
		match self {
			Self::Pending(args) => match args.try_into() {
				Ok([value1, value2, value3]) => Self::Done(f(value1, value2, value3)),
				Err(args) => Self::Pending(args),
			},
			r => r,
		}
	}
}

impl<const NONE: u8, const ONE: u8, const TWO: u8, const THREE: u8>
	ArgsImpl<NONE, ONE, TWO, THREE>
{
	pub fn finish(self, name: &str) -> Result<Value, Error> {
		match self {
			Self::Pending(_) => Err(Error::InvalidArguments {
				name: name.to_owned(),
				message: Self::message(),
			}),
			Self::Done(result) => result,
		}
	}

	fn message() -> String {
		match [NONE == 1, ONE == 1, TWO == 1, THREE == 1] {
			[true, false, false, false] => {
				String::from("The function does not expect any arguments.")
			}
			[false, true, false, false] => String::from("The function expects 1 argument."),
			[false, false, false, false] => unreachable!("some number of arguments must be valid"),
			expectation => {
				let mut message = String::with_capacity(64);
				message.push_str("The function expects ");
				let mut expected = expectation
					.into_iter()
					.enumerate()
					.filter(|(_, e)| *e)
					.map(|(i, _)| i)
					.peekable();
				let mut first = true;
				while let Some(i) = expected.next() {
					if first {
						first = false;
					} else if expected.peek().is_some() {
						message.push_str(", ");
					} else {
						message.push_str(", or ");
					}
					message.push_str(["0", "1", "2", "3"][i]);
				}
				message.push_str(" arguments.");
				message
			}
		}
	}
}

pub trait ArgFrom {
	fn arg_from(value: Value) -> Self;
}

impl ArgFrom for Value {
	fn arg_from(value: Value) -> Self {
		value
	}
}

impl ArgFrom for usize {
	fn arg_from(value: Value) -> Self {
		value.as_int() as usize
	}
}

impl ArgFrom for String {
	fn arg_from(value: Value) -> Self {
		value.as_string()
	}
}
