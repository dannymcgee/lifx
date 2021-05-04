use byteorder::{LittleEndian, WriteBytesExt};
use std::io::Cursor;

use crate::{error::Error, read_write::LittleEndianReader};

/// The Frame section contains information about the following:
///
/// * Size of the entire message
/// * LIFX Protocol number: must be 1024 (decimal)
/// * Use of the Frame Address target field
/// * Source identifier
///
/// The `tagged` field is a boolean that indicates whether the Frame Address target field is
/// being used to address an individual device or all devices.  If `tagged` is true, then the
/// `target` field should be all zeros.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Frame {
	/// 16 bits: Size of entire message in bytes including this field
	pub size: u16,

	/// 2 bits: Message origin indicator: must be zero (0)
	pub origin: u8,

	/// 1 bit: Determines usage of the Frame Address target field
	pub tagged: bool,

	/// 1 bit: Message includes a target address: must be one (1)
	pub addressable: bool,

	/// 12 bits: Protocol number: must be 1024 (decimal)
	pub protocol: u16,

	/// 32 bits: Source identifier: unique value set by the client, used by responses.
	///
	/// If the source identifier is zero, then the LIFX device may send a broadcast message that can
	/// be received by all clients on the same subnet.
	///
	/// If this packet is a reply, then this source field will be set to the same value as the client-
	/// sent request packet.
	pub source: u32,
}

/// The Frame Address section contains the following routing information:
///
/// * Target device address
/// * Acknowledgement message is required flag
/// * State response message is required flag
/// * Message sequence number
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FrameAddress {
	/// 64 bits: 6 byte device address (MAC address) or zero (0) means all devices
	pub target: u64,

	/// 48 bits: Must all be zero (0)
	pub reserved: [u8; 6],

	/// 6 bits: Reserved
	pub reserved2: u8,

	/// 1 bit: Acknowledgement message required
	pub ack_required: bool,

	/// 1 bit: Response message required
	pub res_required: bool,

	/// 8 bits: Wrap around message sequence number
	pub sequence: u8,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ProtocolHeader {
	/// 64 bits: Reserved
	pub reserved: u64,

	/// 16 bits: Message type determines the payload being used
	pub typ: u16,

	/// 16 bits: Reserved
	pub reserved2: u16,
}

impl Frame {
	/// packed sized, in bytes
	pub(crate) fn packed_size() -> usize {
		8
	}

	#[allow(clippy::bool_assert_comparison)]
	pub(crate) fn validate(&self) {
		assert!(self.origin < 4);
		assert_eq!(self.addressable, true);
		assert_eq!(self.protocol, 1024);
	}

	pub(crate) fn pack(&self) -> Result<Vec<u8>, Error> {
		let mut v = Vec::with_capacity(Self::packed_size());

		v.write_u16::<LittleEndian>(self.size)?;

		// pack origin + tagged + addressable +  protocol as a u16
		let mut d: u16 = (<u16 as From<u8>>::from(self.origin) & 0b11) << 14;
		d += if self.tagged { 1 } else { 0 } << 13;
		d += if self.addressable { 1 } else { 0 } << 12;
		d += (self.protocol & 0b1111_1111_1111) as u16;

		v.write_u16::<LittleEndian>(d)?;

		v.write_u32::<LittleEndian>(self.source)?;

		Ok(v)
	}

	pub(crate) fn unpack(v: &[u8]) -> Result<Frame, Error> {
		let mut c = Cursor::new(v);

		let size = c.read_val()?;

		// origin + tagged + addressable + protocol
		let d: u16 = c.read_val()?;

		let origin: u8 = ((d & 0b1100_0000_0000_0000) >> 14) as u8;
		let tagged: bool = (d & 0b0010_0000_0000_0000) > 0;
		let addressable = (d & 0b0001_0000_0000_0000) > 0;
		let protocol: u16 = d & 0b0000_1111_1111_1111;

		if protocol != 1024 {
			return Err(Error::ProtocolError(format!(
				"Unpacked frame had protocol version {}",
				protocol
			)));
		}

		let source = c.read_val()?;

		let frame = Frame {
			size,
			origin,
			tagged,
			addressable,
			protocol,
			source,
		};
		Ok(frame)
	}
}

impl FrameAddress {
	pub(crate) fn packed_size() -> usize {
		16
	}

	pub(crate) fn validate(&self) {
		//assert_eq!(self.reserved, [0;6]);
		//assert_eq!(self.reserved2, 0);
	}

	pub(crate) fn pack(&self) -> Result<Vec<u8>, Error> {
		let mut v = Vec::with_capacity(Self::packed_size());
		v.write_u64::<LittleEndian>(self.target)?;
		for idx in 0..6 {
			v.write_u8(self.reserved[idx])?;
		}

		let b: u8 = (self.reserved2 << 2)
			+ if self.ack_required { 2 } else { 0 }
			+ if self.res_required { 1 } else { 0 };
		v.write_u8(b)?;
		v.write_u8(self.sequence)?;
		Ok(v)
	}

	pub(crate) fn unpack(v: &[u8]) -> Result<FrameAddress, Error> {
		let mut c = Cursor::new(v);

		let target = c.read_val()?;

		let mut reserved: [u8; 6] = [0; 6];
		for slot in &mut reserved {
			*slot = c.read_val()?;
		}

		let b: u8 = c.read_val()?;
		let reserved2: u8 = (b & 0b1111_1100) >> 2;
		let ack_required = (b & 0b10) > 0;
		let res_required = (b & 0b01) > 0;

		let sequence = c.read_val()?;

		let f = FrameAddress {
			target,
			reserved,
			reserved2,
			ack_required,
			res_required,
			sequence,
		};
		f.validate();
		Ok(f)
	}
}

impl ProtocolHeader {
	pub(crate) fn packed_size() -> usize {
		12
	}

	pub(crate) fn validate(&self) {
		//assert_eq!(self.reserved, 0);
		//assert_eq!(self.reserved2, 0);
	}

	/// Packs this part of the packet into some bytes
	pub fn pack(&self) -> Result<Vec<u8>, Error> {
		let mut v = Vec::with_capacity(Self::packed_size());
		v.write_u64::<LittleEndian>(self.reserved)?;
		v.write_u16::<LittleEndian>(self.typ)?;
		v.write_u16::<LittleEndian>(self.reserved2)?;
		Ok(v)
	}

	pub(crate) fn unpack(v: &[u8]) -> Result<ProtocolHeader, Error> {
		let mut c = Cursor::new(v);

		let reserved = c.read_val()?;
		let typ = c.read_val()?;
		let reserved2 = c.read_val()?;

		let f = ProtocolHeader {
			reserved,
			typ,
			reserved2,
		};
		f.validate();
		Ok(f)
	}
}
