use std::io;
use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};

pub trait LittleEndianReader<T> {
	fn read_val(&mut self) -> Result<T, io::Error>;
}
impl<R: ReadBytesExt> LittleEndianReader<u8> for R {
	fn read_val(&mut self) -> Result<u8, io::Error> {
		self.read_u8()
	}
}
macro_rules! derive_reader {
{ $( $m:ident: $t:ty ),*} => {
		$(
			impl<T: ReadBytesExt> LittleEndianReader<$t> for T {
				fn read_val(&mut self) -> Result<$t, io::Error> {
						self . $m ::<LittleEndian>()
				}
			}
		)*

}
}
derive_reader! { read_u32: u32, read_u16: u16, read_i16: i16, read_u64: u64, read_f32: f32 }

pub trait LittleEndianWriter<T>: WriteBytesExt {
	fn write_val(&mut self, v: T) -> Result<(), io::Error>;
}
impl<T: WriteBytesExt> LittleEndianWriter<u8> for T {
	fn write_val(&mut self, v: u8) -> Result<(), io::Error> {
		self.write_u8(v)
	}
}
impl<T: WriteBytesExt> LittleEndianWriter<bool> for T {
	fn write_val(&mut self, v: bool) -> Result<(), io::Error> {
		self.write_u8(if v { 1 } else { 0 })
	}
}
macro_rules! derive_writer {
{ $( $m:ident: $t:ty ),*} => {
	$(
		impl<T: WriteBytesExt> LittleEndianWriter<$t> for T {
			fn write_val(&mut self, v: $t) -> Result<(), io::Error> {
				self . $m ::<LittleEndian>(v)
			}
		}
	)*

}
}
derive_writer! { write_u32: u32, write_u16: u16, write_i16: i16, write_u64: u64, write_f32: f32 }
