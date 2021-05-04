use std::io;
use byteorder::{ReadBytesExt, WriteBytesExt};

use crate::read_write::{LittleEndianReader, LittleEndianWriter};

/// Lifx strings are fixed-length (32-bytes maximum)
#[derive(Debug, Clone, PartialEq)]
pub struct LifxString(pub String);

impl LifxString {
	/// Constructs a new LifxString, truncating to 32 characters.
	pub fn new(s: &str) -> LifxString {
		LifxString(if s.len() > 32 {
			s[..32].to_owned()
		} else {
			s.to_owned()
		})
	}
}

impl std::fmt::Display for LifxString {
	fn fmt(&self, fmt: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
		write!(fmt, "{}", self.0)
	}
}

impl std::cmp::PartialEq<str> for LifxString {
	fn eq(&self, other: &str) -> bool {
		self.0 == other
	}
}

impl<R: ReadBytesExt> LittleEndianReader<LifxString> for R {
	fn read_val(&mut self) -> Result<LifxString, io::Error> {
		let mut label = String::with_capacity(32);
		for _ in 0..32 {
			let c: u8 = self.read_val()?;
			if c > 0 {
				label.push(c as char);
			}
		}
		Ok(LifxString(label))
	}
}


impl<T> LittleEndianWriter<LifxString> for T
where
	T: WriteBytesExt,
{
	fn write_val(&mut self, v: LifxString) -> Result<(), io::Error> {
		for idx in 0..32 {
			if idx >= v.0.len() {
				self.write_u8(0)?;
			} else {
				self.write_u8(v.0.chars().nth(idx).unwrap() as u8)?;
			}
		}
		Ok(())
	}
}
