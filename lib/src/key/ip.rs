use derive::Key;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Eq, PartialEq, PartialOrd, Serialize, Deserialize, Key)]
pub struct Ip<'a> {
	__: u8,
	_a: u8,
	_b: u8,
	_c: u8,
	pub ip: &'a str,
}

pub fn new(bu: &str) -> Ns<'_> {
	Ip::new(bu)
}

pub fn prefix() -> Vec<u8> {
	let mut k = super::kv::new().encode().unwrap();
	k.extend_from_slice(&[b'!', b'i', b'p', 0x00]);
	k
}

pub fn suffix() -> Vec<u8> {
	let mut k = super::kv::new().encode().unwrap();
	k.extend_from_slice(&[b'!', b'i', b'p', 0xff]);
	k
}

impl<'a> Ip<'a> {
	pub fn new(ip: &'a str) -> Self {
		Self {
			__: b'/',
			_a: b'!',
			_b: b'l',
			_c: b'm',
			ip,
		}
	}
}

#[cfg(test)]
mod tests {
	#[test]
	fn key() {
		use super::*;
		#[rustfmt::skip]
		let val = Ip::new(
			"1.2.3.4",
		);
		let enc = Ip::encode(&val).unwrap();
		let dec = Ip::decode(&enc).unwrap();
		assert_eq!(val, dec);
	}
}
