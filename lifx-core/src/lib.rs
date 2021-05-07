//! This crate provides low-level message types and structures for dealing with the LIFX LAN protocol.
//!
//! This lets you control lights on your local area network.  More info can be found here:
//! https://lan.developer.lifx.com/
//!
//! Since this is a low-level library, it does not deal with issues like talking to the network,
//! caching light state, or waiting for replies.  This should be done at a higher-level library.
//!
//! # Discovery
//!
//! To discover lights on your LAN, send a [Message::GetService] message as a UDP broadcast to port 56700
//! When a device is discovered, the [Service] types and IP port are provided.  To get additional
//! info about each device, send additional Get messages directly to each device (by setting the
//! [FrameAddress::target] field to the bulbs target ID, and then send a UDP packet to the IP address
//! associated with the device).
//!
//! # Reserved fields
//! When *constructing* packets, you must always set every reserved field to zero.  However, it's
//! possible to receive packets with these fields set to non-zero values.  Be conservative in what
//! you send, and liberal in what you accept.
//!
//! # Unknown values
//! It's common to see packets for LIFX bulbs that don't match the documented protocol.  These are
//! suspected to be internal messages that are used by offical LIFX apps, but that aren't documented.

#![allow(clippy::bool_assert_comparison)]
#![feature(exclusive_range_pattern)]

mod error;
mod string;
mod read_write;
mod msg;
mod protocol;
mod color;
mod misc;
mod product;
pub mod udp;

pub use error::Error;
pub use string::LifxString;
pub use read_write::{LittleEndianReader, LittleEndianWriter};
pub use msg::{BuildOptions, Message, RawMessage};
pub use protocol::{Frame, FrameAddress, ProtocolHeader};
pub use color::{ApplicationRequest, Waveform, HSBK, Kelvin};
pub use misc::{EchoPayload, LifxIdent, PowerLevel, Service};
pub use product::{get_product_info, ProductInfo};

//trace_macros!(true);
//message_types! {
//    /// GetService - 2
//    ///
//    /// Sent by a client to acquire responses from all devices on the local network.
//    GetService(2, ),
//    /// StateService - 3
//    ///
//    /// Response to GetService message.  Provides the device Service and Port.  If the Service
//    /// is temporarily unavailable, then the port value will be zero
//    StateService(3, {
//        service: Service,
//        port: u32
//    })
//}
//trace_macros!(false);

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_frame() {
		let frame = Frame {
			size: 0x1122,
			origin: 0,
			tagged: true,
			addressable: true,
			protocol: 1024,
			source: 1234567,
		};
		frame.validate();

		let v = frame.pack().unwrap();
		println!("{:?}", v);
		assert_eq!(v[0], 0x22);
		assert_eq!(v[1], 0x11);

		assert_eq!(v.len(), Frame::packed_size());

		let unpacked = Frame::unpack(&v).unwrap();
		assert_eq!(frame, unpacked);
	}

	#[test]
	fn test_decode_frame() {
		//             00    01    02    03    04    05    06    07
		let v = vec![0x28, 0x00, 0x00, 0x54, 0x42, 0x52, 0x4b, 0x52];
		let frame = Frame::unpack(&v).unwrap();
		println!("{:?}", frame);

		// manual decoding:
		// size: 0x0028 ==> 40
		// 0x00, 0x54 (origin, tagged, addressable, protocol)

		//  /-Origin ==> 0
		// || /- addressable=1
		// || |
		// 01010100 00000000
		//   |
		//   \- Tagged=0

		assert_eq!(frame.size, 0x0028);
		assert_eq!(frame.origin, 1);
		assert_eq!(frame.addressable, true);
		assert_eq!(frame.tagged, false);
		assert_eq!(frame.protocol, 1024);
		assert_eq!(frame.source, 0x524b5242);
	}

	#[test]
	fn test_decode_frame1() {
		//             00    01    02    03    04    05    06    07
		let v = vec![0x24, 0x00, 0x00, 0x14, 0xca, 0x41, 0x37, 0x05];
		let frame = Frame::unpack(&v).unwrap();
		println!("{:?}", frame);

		// 00010100 00000000

		assert_eq!(frame.size, 0x0024);
		assert_eq!(frame.origin, 0);
		assert_eq!(frame.tagged, false);
		assert_eq!(frame.addressable, true);
		assert_eq!(frame.protocol, 1024);
		assert_eq!(frame.source, 0x053741ca);
	}

	#[test]
	fn test_frame_address() {
		let frame = FrameAddress {
			target: 0x11224488,
			reserved: [0; 6],
			reserved2: 0,
			ack_required: true,
			res_required: false,
			sequence: 248,
		};
		frame.validate();

		let v = frame.pack().unwrap();
		assert_eq!(v.len(), FrameAddress::packed_size());
		println!("Packed FrameAddress: {:?}", v);

		let unpacked = FrameAddress::unpack(&v).unwrap();
		assert_eq!(frame, unpacked);
	}

	#[test]
	fn test_decode_frame_address() {
		//   1  2  3  4  5  6  7  8  9  10 11 12 13 14 15 16
		let v = vec![
			0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
			0x01, 0x9c,
		];
		assert_eq!(v.len(), FrameAddress::packed_size());

		let frame = FrameAddress::unpack(&v).unwrap();
		frame.validate();
		println!("FrameAddress: {:?}", frame);
	}

	#[test]
	fn test_protocol_header() {
		let frame = ProtocolHeader {
			reserved: 0,
			reserved2: 0,
			typ: 0x4455,
		};
		frame.validate();

		let v = frame.pack().unwrap();
		assert_eq!(v.len(), ProtocolHeader::packed_size());
		println!("Packed ProtocolHeader: {:?}", v);

		let unpacked = ProtocolHeader::unpack(&v).unwrap();
		assert_eq!(frame, unpacked);
	}

	#[test]
	fn test_decode_protocol_header() {
		//   1  2  3  4  5  6  7  8  9  10 11 12 13 14 15 16
		let v = vec![
			0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x0e, 0x00, 0x00, 0x00,
		];
		assert_eq!(v.len(), ProtocolHeader::packed_size());

		let frame = ProtocolHeader::unpack(&v).unwrap();
		frame.validate();
		println!("ProtocolHeader: {:?}", frame);
	}

	#[test]
	fn test_decode_full() {
		let v = vec![
			0x24, 0x00, 0x00, 0x14, 0xca, 0x41, 0x37, 0x05, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
			0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01, 0x98, 0x00, 0x00, 0x00, 0x00,
			0x00, 0x00, 0x00, 0x00, 0x33, 0x00, 0x00, 0x00,
		];

		let msg = RawMessage::unpack(&v).unwrap();
		msg.validate();
		println!("{:#?}", msg);
	}

	#[test]
	fn test_decode_full_1() {
		let v = vec![
			0x58, 0x00, 0x00, 0x54, 0xca, 0x41, 0x37, 0x05, 0xd0, 0x73, 0xd5, 0x02, 0x97, 0xde,
			0x00, 0x00, 0x4c, 0x49, 0x46, 0x58, 0x56, 0x32, 0x00, 0xc0, 0x44, 0x30, 0xeb, 0x47,
			0xc4, 0x48, 0x18, 0x14, 0x6b, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xff, 0xff,
			0xb8, 0x0b, 0x00, 0x00, 0xff, 0xff, 0x4b, 0x69, 0x74, 0x63, 0x68, 0x65, 0x6e, 0x00,
			0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
			0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
			0x00, 0x00, 0x00, 0x00,
		];

		let msg = RawMessage::unpack(&v).unwrap();
		msg.validate();
		println!("{:#?}", msg);
	}

	#[test]
	fn test_build_a_packet() {
		// packet taken from https://lan.developer.lifx.com/docs/building-a-lifx-packet

		let msg = Message::LightSetColor {
			reserved: 0,
			color: HSBK {
				hue: 21845,
				saturation: 0xffff,
				brightness: 0xffff,
				kelvin: 3500,
			},
			duration: 1024,
		};

		let raw = RawMessage::build(
			&BuildOptions {
				target: None,
				ack_required: false,
				res_required: false,
				sequence: 0,
				source: 0,
			},
			msg,
		)
		.unwrap();

		let bytes = raw.pack().unwrap();
		println!("{:?}", bytes);
		assert_eq!(bytes.len(), 49);
		assert_eq!(
			bytes,
			vec![
				0x31, 0x00, 0x00, 0x34, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
				0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
				0x00, 0x00, 0x00, 0x00, 0x66, 0x00, 0x00, 0x00, 0x00, 0x55, 0x55, 0xFF, 0xFF, 0xFF,
				0xFF, 0xAC, 0x0D, 0x00, 0x04, 0x00, 0x00
			]
		);
	}
}
