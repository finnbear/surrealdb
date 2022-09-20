use std::ops::Div;
use crate::sql::number::Number;

pub trait Median {
	fn median(&mut self) -> Number;
}

impl Median for Vec<Number> {
	fn median(&mut self) -> Number {
		self.sort();
		if self.len() % 2 == 0 {
			(self.remove(self.len() / 2) + self.remove(self.len() / 2)).div() / Number::Int(2)
		} else {
			self.remove(self.len() / 2)
		}
	}
}
