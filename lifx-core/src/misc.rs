use std::{io, convert::TryFrom};
use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};

use crate::{
	error::Error,
	read_write::{LittleEndianReader, LittleEndianWriter},
};

#[derive(Debug, Clone, PartialEq)]
pub struct LifxIdent(pub [u8; 16]);

impl<R: ReadBytesExt> LittleEndianReader<LifxIdent> for R {
	fn read_val(&mut self) -> Result<LifxIdent, io::Error> {
		let mut val = [0; 16];
		for v in &mut val {
			*v = self.read_val()?;
		}
		Ok(LifxIdent(val))
	}
}

impl<T> LittleEndianWriter<LifxIdent> for T
where
	T: WriteBytesExt,
{
	fn write_val(&mut self, v: LifxIdent) -> Result<(), io::Error> {
		for idx in 0..16 {
			self.write_u8(v.0[idx])?;
		}
		Ok(())
	}
}

#[derive(Copy, Clone)]
pub struct EchoPayload(pub [u8; 64]);

impl std::fmt::Debug for EchoPayload {
	fn fmt(&self, f: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
		write!(f, "<EchoPayload>")
	}
}

impl<R: ReadBytesExt> LittleEndianReader<EchoPayload> for R {
	fn read_val(&mut self) -> Result<EchoPayload, io::Error> {
		let mut val = [0; 64];
		for v in val.iter_mut() {
			*v = self.read_val()?;
		}
		Ok(EchoPayload(val))
	}
}

impl<T> LittleEndianWriter<EchoPayload> for T
where
	T: WriteBytesExt,
{
	fn write_val(&mut self, v: EchoPayload) -> Result<(), io::Error> {
		for idx in 0..64 {
			self.write_u8(v.0[idx])?;
		}
		Ok(())
	}
}

#[repr(u16)]
#[derive(Debug, Copy, Clone, PartialEq)]
pub enum PowerLevel {
	Standby = 0,
	Enabled = 65535,
}

impl<T> LittleEndianWriter<PowerLevel> for T
where
	T: WriteBytesExt,
{
	fn write_val(&mut self, v: PowerLevel) -> Result<(), io::Error> {
		self.write_u16::<LittleEndian>(v as u16)
	}
}

impl TryFrom<u16> for PowerLevel {
	type Error = Error;
	fn try_from(val: u16) -> Result<PowerLevel, Error> {
		match val {
			x if x == PowerLevel::Enabled as u16 => Ok(PowerLevel::Enabled),
			x if x == PowerLevel::Standby as u16 => Ok(PowerLevel::Standby),
			x => Err(Error::ProtocolError(format!("Unknown power level {}", x))),
		}
	}
}

/// What services are exposed by the device.
///
/// LIFX only documents the UDP service, though bulbs may support other undocumented services.
/// Since these other services are unsupported by the lifx-core library, a message with a non-UDP
/// service cannot be constructed.
#[repr(u8)]
#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Service {
	UDP = 1,
}

impl TryFrom<u8> for Service {
	type Error = Error;
	fn try_from(val: u8) -> Result<Service, Error> {
		if val != Service::UDP as u8 {
			Err(Error::ProtocolError(format!(
				"Unknown service value {}",
				val
			)))
		} else {
			Ok(Service::UDP)
		}
	}
}
