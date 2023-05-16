use super::mean::Mean;
use crate::err::Error;
use crate::sql::number::TrySum;
use crate::sql::value::{TryMul, TryDiv};
use crate::sql::{number::Number, value::TrySub};

pub trait Variance {
	/// Population Variance of Data
	/// O(n) time complex
	fn variance(self, sample: bool) -> Number;
}

impl Variance for Vec<Number> {
	fn variance(self, sample: bool) -> Number {
		match self.len() {
			0 => Number::NAN,
			1 => Number::from(0),
			len => {
				let mean = self.mean();
				let len = Number::from(len - sample as usize);
				let out = Number::try_sum(
					self.iter().map(|x| x.try_sub(&mean)?.try_mul(x.try_sub(&mean)?)?),
				)
				.unwrap_or(Number::NAN)
				.try_div(len)
				.unwrap_or(Number::NAN);
				out
			}
		}
	}
}
