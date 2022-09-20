use std::collections::BinaryHeap;
use crate::sql::number::Number;

pub trait Top {
	fn top(self, _c: i64) -> Number;
}

impl Top for Vec<Number> {
	fn top(mut self, c: i64) -> Number {
		self.sort();
		self.into_iter().nth(self.len() - c as usize).unwrap()
	}
}
