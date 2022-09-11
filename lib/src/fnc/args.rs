use std::future;
use std::future::Future;
use crate::ctx::Context;
use crate::err::Error;
use crate::sql::value::Value;

pub trait ArgCast: Sized {
	fn from_value(value: Value) -> Result<Self, Error>;
}

impl ArgCast for Value {
	fn from_value(value: Value) -> Result<Self, Error> {
		Ok(value)
	}
}

impl ArgCast for String {
	fn from_value(value: Value) -> Result<Self, Error> {
		Ok(value.as_string())
	}
}

impl ArgCast for usize {
	fn from_value(value: Value) -> Result<Self, Error> {
		Ok(value.as_int() as usize)
	}
}

pub trait Function {
	type Fut: Future<Output = Result<Value, Error>>;
	fn call(&self, name: &str, context: &Context, args: Vec<Value>) -> Self::Fut;
}

impl<F: Fn(Value) -> Result<Value, Error>> Function for F {
	type Fut = future::Ready<Result<Value, Error>>;

	fn call(&self, name: &str, _: &Context, args: Vec<Value>) -> Self::Fut {
		future::ready(match <Vec<Value> as TryInto<[Value; 1]>>::try_into(args) {
			Ok([arg]) => self(arg),
			Err(_) => Err(Error::InvalidArguments {
				name: name.to_owned(),
				message: format!("Expected 1 argument.")
			})
		})
	}
}

pub enum Args {
	None,
	Any,
	One,
	Two,
	Three,
	NoneOne,
	NoneTwo,
	NoneOneTwo,
	OneTwo,
}

pub fn check(
	ctx: &Context,
	name: &str,
	args: Vec<Value>,
	size: Args,
	func: fn(&Context, Vec<Value>) -> Result<Value, Error>,
) -> Result<Value, Error> {
	match size {
		Args::None => match args.len() {
			0 => func(ctx, args),
			_ => Err(Error::InvalidArguments {
				name: name.to_owned(),
				message: String::from("The function does not expect any arguments."),
			}),
		},
		Args::One => match args.len() {
			1 => func(ctx, args),
			_ => Err(Error::InvalidArguments {
				name: name.to_owned(),
				message: String::from("The function expects 1 argument."),
			}),
		},
		Args::Two => match args.len() {
			2 => func(ctx, args),
			_ => Err(Error::InvalidArguments {
				name: name.to_owned(),
				message: String::from("The function expects 2 arguments."),
			}),
		},
		Args::Three => match args.len() {
			3 => func(ctx, args),
			_ => Err(Error::InvalidArguments {
				name: name.to_owned(),
				message: String::from("The function expects 3 arguments."),
			}),
		},
		Args::NoneOne => match args.len() {
			0 | 1 => func(ctx, args),
			_ => Err(Error::InvalidArguments {
				name: name.to_owned(),
				message: String::from("The function expects 0 or 1 arguments."),
			}),
		},
		Args::NoneTwo => match args.len() {
			0 | 2 => func(ctx, args),
			_ => Err(Error::InvalidArguments {
				name: name.to_owned(),
				message: String::from("The function expects 0 or 2 arguments."),
			}),
		},
		Args::NoneOneTwo => match args.len() {
			0 | 1 | 2 => func(ctx, args),
			_ => Err(Error::InvalidArguments {
				name: name.to_owned(),
				message: String::from("The function expects 0, 1, or 2 arguments."),
			}),
		},
		Args::OneTwo => match args.len() {
			1 | 2 => func(ctx, args),
			_ => Err(Error::InvalidArguments {
				name: name.to_owned(),
				message: String::from("The function expects 1 or 2 arguments."),
			}),
		},
		Args::Any => func(ctx, args),
	}
}
