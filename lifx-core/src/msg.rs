use std::{convert::TryInto, io::Cursor};

use crate::{
	color::{ApplicationRequest, HSBK, Waveform},
	error::Error,
	protocol::{Frame, FrameAddress, ProtocolHeader},
	read_write::{LittleEndianReader, LittleEndianWriter},
	string::LifxString,
	misc::{EchoPayload, LifxIdent, PowerLevel, Service},
};

macro_rules! unpack {
	($msg:ident, $typ:ident, $( $n:ident: $t:ident ),*) => {
		 {
		 let mut c = Cursor::new(&$msg.payload);
		 $(
			  let $n: $t = c.read_val()?;
		 )*

			  Message::$typ {
			  $(
						 $n: $n.try_into()?,
			  )*
		 }
		 }
	};
}

/// Options used to contruct a [RawMessage].
///
/// See also [RawMessage::build].
#[derive(Debug, Clone)]
pub struct BuildOptions {
	/// If not `None`, this is the ID of the device you want to address.
	///
	/// To look up the ID of a device, extract it from the [FrameAddress::target] field when a
	/// device sends a [Message::StateService] message.
	pub target: Option<u64>,
	/// Acknowledgement message required.
	///
	/// Causes the light to send an [Message::Acknowledgement] message.
	pub ack_required: bool,
	/// Response message required.
	///
	/// Some message types are sent by clients to get data from a light.  These should always have
	/// `res_required` set to true.
	pub res_required: bool,
	/// A wrap around sequence number.  Optional (can be zero).
	///
	/// By providing a unique sequence value, the response message will also contain the same
	/// sequence number, allowing a client to distinguish between different messages sent with the
	/// same `source` identifier.
	pub sequence: u8,
	/// A unique client identifier. Optional (can be zero).
	///
	/// If the source is non-zero, then the LIFX device with send a unicast message to the IP
	/// address/port of the client that sent the originating message.  If zero, then the LIFX
	/// device may send a broadcast message that can be received by all clients on the same sub-net.
	pub source: u32,
}

impl std::default::Default for BuildOptions {
	fn default() -> BuildOptions {
		BuildOptions {
			target: None,
			ack_required: false,
			res_required: false,
			sequence: 0,
			source: 0,
		}
	}
}

impl RawMessage {
	/// Build a RawMessage (which is suitable for sending on the network) from a given Message
	/// type.
	///
	/// If [BuildOptions::target] is None, then the message is addressed to all devices.  Else it should be a
	/// bulb UID (MAC address)
	pub fn build(options: &BuildOptions, typ: Message) -> Result<RawMessage, Error> {
		let frame = Frame {
			size: 0,
			origin: 0,
			tagged: options.target.is_none(),
			addressable: true,
			protocol: 1024,
			source: options.source,
		};
		let addr = FrameAddress {
			target: options.target.unwrap_or(0),
			reserved: [0; 6],
			reserved2: 0,
			ack_required: options.ack_required,
			res_required: options.res_required,
			sequence: options.sequence,
		};
		let phead = ProtocolHeader {
			reserved: 0,
			reserved2: 0,
			typ: typ.get_num(),
		};

		let mut v = Vec::new();
		match typ {
			Message::GetService
			| Message::GetHostInfo
			| Message::GetHostFirmware
			| Message::GetWifiFirmware
			| Message::GetWifiInfo
			| Message::GetPower
			| Message::GetLabel
			| Message::GetVersion
			| Message::GetInfo
			| Message::Acknowledgement { .. }
			| Message::GetLocation
			| Message::GetGroup
			| Message::LightGet
			| Message::LightGetPower
			| Message::LightGetInfrared => {
				// these types have no payload
			}
			Message::SetColorZones {
				start_index,
				end_index,
				color,
				duration,
				apply,
			} => {
				v.write_val(start_index)?;
				v.write_val(end_index)?;
				v.write_val(color)?;
				v.write_val(duration)?;
				v.write_val(apply)?;
			}
			Message::SetWaveform {
				reserved,
				transient,
				color,
				period,
				cycles,
				skew_ratio,
				waveform,
			} => {
				v.write_val(reserved)?;
				v.write_val(transient)?;
				v.write_val(color)?;
				v.write_val(period)?;
				v.write_val(cycles)?;
				v.write_val(skew_ratio)?;
				v.write_val(waveform)?;
			}
			Message::SetWaveformOptional {
				reserved,
				transient,
				color,
				period,
				cycles,
				skew_ratio,
				waveform,
				set_hue,
				set_saturation,
				set_brightness,
				set_kelvin,
			} => {
				v.write_val(reserved)?;
				v.write_val(transient)?;
				v.write_val(color)?;
				v.write_val(period)?;
				v.write_val(cycles)?;
				v.write_val(skew_ratio)?;
				v.write_val(waveform)?;
				v.write_val(set_hue)?;
				v.write_val(set_saturation)?;
				v.write_val(set_brightness)?;
				v.write_val(set_kelvin)?;
			}
			Message::GetColorZones {
				start_index,
				end_index,
			} => {
				v.write_val(start_index)?;
				v.write_val(end_index)?;
			}
			Message::StateZone {
				count,
				index,
				color,
			} => {
				v.write_val(count)?;
				v.write_val(index)?;
				v.write_val(color)?;
			}
			Message::StateMultiZone {
				count,
				index,
				color0,
				color1,
				color2,
				color3,
				color4,
				color5,
				color6,
				color7,
			} => {
				v.write_val(count)?;
				v.write_val(index)?;
				v.write_val(color0)?;
				v.write_val(color1)?;
				v.write_val(color2)?;
				v.write_val(color3)?;
				v.write_val(color4)?;
				v.write_val(color5)?;
				v.write_val(color6)?;
				v.write_val(color7)?;
			}
			Message::LightStateInfrared { brightness } => v.write_val(brightness)?,
			Message::LightSetInfrared { brightness } => v.write_val(brightness)?,
			Message::SetLocation {
				location,
				label,
				updated_at,
			} => {
				v.write_val(location)?;
				v.write_val(label)?;
				v.write_val(updated_at)?;
			}
			Message::SetGroup {
				group,
				label,
				updated_at,
			} => {
				v.write_val(group)?;
				v.write_val(label)?;
				v.write_val(updated_at)?;
			}
			Message::StateService { port, service } => {
				v.write_val(port)?;
				v.write_val(service as u8)?;
			}
			Message::StateHostInfo {
				signal,
				tx,
				rx,
				reserved,
			} => {
				v.write_val(signal)?;
				v.write_val(tx)?;
				v.write_val(rx)?;
				v.write_val(reserved)?;
			}
			Message::StateHostFirmware {
				build,
				reserved,
				version,
			} => {
				v.write_val(build)?;
				v.write_val(reserved)?;
				v.write_val(version)?;
			}
			Message::StateWifiInfo {
				signal,
				tx,
				rx,
				reserved,
			} => {
				v.write_val(signal)?;
				v.write_val(tx)?;
				v.write_val(rx)?;
				v.write_val(reserved)?;
			}
			Message::StateWifiFirmware {
				build,
				reserved,
				version,
			} => {
				v.write_val(build)?;
				v.write_val(reserved)?;
				v.write_val(version)?;
			}
			Message::SetPower { level } => {
				v.write_val(level)?;
			}
			Message::StatePower { level } => {
				v.write_val(level)?;
			}
			Message::SetLabel { label } => {
				v.write_val(label)?;
			}
			Message::StateLabel { label } => {
				v.write_val(label)?;
			}
			Message::StateVersion {
				vendor,
				product,
				version,
			} => {
				v.write_val(vendor)?;
				v.write_val(product)?;
				v.write_val(version)?;
			}
			Message::StateInfo {
				time,
				uptime,
				downtime,
			} => {
				v.write_val(time)?;
				v.write_val(uptime)?;
				v.write_val(downtime)?;
			}
			Message::StateLocation {
				location,
				label,
				updated_at,
			} => {
				v.write_val(location)?;
				v.write_val(label)?;
				v.write_val(updated_at)?;
			}
			Message::StateGroup {
				group,
				label,
				updated_at,
			} => {
				v.write_val(group)?;
				v.write_val(label)?;
				v.write_val(updated_at)?;
			}
			Message::EchoRequest { payload } => {
				v.write_val(payload)?;
			}
			Message::EchoResponse { payload } => {
				v.write_val(payload)?;
			}
			Message::LightSetColor {
				reserved,
				color,
				duration,
			} => {
				v.write_val(reserved)?;
				v.write_val(color)?;
				v.write_val(duration)?;
			}
			Message::LightState {
				color,
				reserved,
				power,
				label,
				reserved2,
			} => {
				v.write_val(color)?;
				v.write_val(reserved)?;
				v.write_val(power)?;
				v.write_val(label)?;
				v.write_val(reserved2)?;
			}
			Message::LightSetPower { level, duration } => {
				v.write_val(if level > 0 { 65535u16 } else { 0u16 })?;
				v.write_val(duration)?;
			}
			Message::LightStatePower { level } => {
				v.write_val(level)?;
			}
		}

		let mut msg = RawMessage {
			frame,
			frame_addr: addr,
			protocol_header: phead,
			payload: v,
		};

		msg.frame.size = msg.packed_size() as u16;

		Ok(msg)
	}

	/// The total size (in bytes) of the packed version of this message.
	pub fn packed_size(&self) -> usize {
		Frame::packed_size()
			+ FrameAddress::packed_size()
			+ ProtocolHeader::packed_size()
			+ self.payload.len()
	}

	/// Validates that this object was constructed correctly.  Panics if not.
	pub fn validate(&self) {
		self.frame.validate();
		self.frame_addr.validate();
		self.protocol_header.validate();
	}

	/// Packs this RawMessage into some bytes that can be send over the network.
	///
	/// The length of the returned data will be [RawMessage::packed_size] in size.
	pub fn pack(&self) -> Result<Vec<u8>, Error> {
		let mut v = Vec::with_capacity(self.packed_size());
		v.extend(self.frame.pack()?);
		v.extend(self.frame_addr.pack()?);
		v.extend(self.protocol_header.pack()?);
		v.extend(&self.payload);
		Ok(v)
	}
	/// Given some bytes (generally read from a network socket), unpack the data into a
	/// `RawMessage` structure.
	pub fn unpack(v: &[u8]) -> Result<RawMessage, Error> {
		let mut start = 0;
		let frame = Frame::unpack(v)?;
		frame.validate();
		start += Frame::packed_size();
		let addr = FrameAddress::unpack(&v[start..])?;
		addr.validate();
		start += FrameAddress::packed_size();
		let proto = ProtocolHeader::unpack(&v[start..])?;
		proto.validate();
		start += ProtocolHeader::packed_size();

		let body = Vec::from(&v[start..(frame.size as usize)]);

		Ok(RawMessage {
			frame,
			frame_addr: addr,
			protocol_header: proto,
			payload: body,
		})
	}
}

/// The raw message structure
///
/// Contains a low-level protocol info.  This is what is sent and received via UDP packets.
///
/// To parse the payload, use [Message::from_raw].
#[derive(Debug, Clone, PartialEq)]
pub struct RawMessage {
	pub frame: Frame,
	pub frame_addr: FrameAddress,
	pub protocol_header: ProtocolHeader,
	pub payload: Vec<u8>,
}

/// Decoded LIFX Messages
///
/// This enum lists all of the LIFX message types known to this library.
///
/// Note that other message types exist, but are not officially documented (and so are not
/// available here).
#[derive(Clone, Debug)]
pub enum Message {
	/// GetService - 2
	///
	/// Sent by a client to acquire responses from all devices on the local network. No payload is
	/// required. Causes the devices to transmit a StateService message.
	GetService,

	/// StateService - 3
	///
	/// Response to [Message::GetService] message.
	StateService {
		/// Port number of the light.  If the service is temporarily unavailable, then the port value
		/// will be 0.
		port: u32,
		/// unsigned 8-bit integer, maps to `Service`
		service: Service,
	},

	/// GetHostInfo - 12
	///
	/// Get Host MCU information. No payload is required. Causes the device to transmit a
	/// [Message::StateHostInfo] message.
	GetHostInfo,

	/// StateHostInfo - 13
	///
	/// Response to [Message::GetHostInfo] message.
	///
	/// Provides host MCU information.
	StateHostInfo {
		/// radio receive signal strength in miliWatts
		signal: f32,
		/// Bytes transmitted since power on
		tx: u32,
		/// Bytes received since power on
		rx: u32,
		reserved: i16,
	},

	/// GetHostFirmware - 14
	///
	/// Gets Host MCU firmware information. No payload is required. Causes the device to transmit a
	/// [Message::StateHostFirmware] message.
	GetHostFirmware,

	/// StateHostFirmware - 15
	///
	/// Response to [Message::GetHostFirmware] message.
	///
	/// Provides host firmware information.
	StateHostFirmware {
		/// Firmware build time (absolute time in nanoseconds since epoch)
		build: u64,
		reserved: u64,
		/// Firmware version
		version: u32,
	},

	/// GetWifiInfo - 16
	///
	/// Get Wifi subsystem information. No payload is required. Causes the device to transmit a
	/// [Message::StateWifiInfo] message.
	GetWifiInfo,

	/// StateWifiInfo - 17
	///
	/// Response to [Message::GetWifiInfo] message.
	///
	/// Provides Wifi subsystem information.
	StateWifiInfo {
		/// Radio receive signal strength in mw
		signal: f32,
		/// bytes transmitted since power on
		tx: u32,
		/// bytes received since power on
		rx: u32,
		reserved: i16,
	},

	/// GetWifiFirmware - 18
	///
	/// Get Wifi subsystem firmware. No payload is required. Causes the device to transmit a
	/// [Message::StateWifiFirmware] message.
	GetWifiFirmware,

	/// StateWifiFirmware - 19
	/// \
	/// Response to [Message::GetWifiFirmware] message.
	///
	/// Provides Wifi subsystem information.
	StateWifiFirmware {
		/// firmware build time (absolute time in nanoseconds since epoch)
		build: u64,
		reserved: u64,
		/// firmware version
		version: u32,
	},

	/// GetPower - 20
	///
	/// Get device power level. No payload is required. Causes the device to transmit a [Message::StatePower]
	/// message
	GetPower,

	/// SetPower - 21
	///
	/// Set device power level.
	SetPower {
		/// normally a u16, but only 0 and 65535 are supported.
		///
		/// Zero implies standby and non-zero sets a corresponding power draw level.
		level: PowerLevel,
	},

	/// StatePower - 22
	///
	/// Response to [Message::GetPower] message.
	///
	/// Provides device power level.
	StatePower { level: PowerLevel },

	/// GetLabel - 23
	///
	/// Get device label. No payload is required. Causes the device to transmit a [Message::StateLabel]
	/// message.
	GetLabel,

	/// SetLabel - 24
	///
	/// Set the device label text.
	SetLabel { label: LifxString },

	/// StateLabel - 25
	///
	/// Response to [Message::GetLabel] message.
	///
	/// Provides device label.
	StateLabel { label: LifxString },

	/// GetVersion - 32
	///
	/// Get the hardware version. No payload is required. Causes the device to transmit a
	/// [Message::StateVersion] message.
	GetVersion,

	/// StateVersion - 33
	///
	/// Response to [Message::GetVersion] message.
	///
	/// Provides the hardware version of the device.
	StateVersion {
		/// vendor ID
		vendor: u32,
		/// product ID
		product: u32,
		/// hardware version
		version: u32,
	},

	/// GetInfo - 34
	///
	/// Get run-time information. No payload is required. Causes the device to transmit a [Message::StateInfo]
	/// message.
	GetInfo,

	/// StateInfo - 35
	///
	/// Response to [Message::GetInfo] message.
	///
	/// Provides run-time information of device.
	StateInfo {
		/// current time (absolute time in nanoseconds since epoch)
		time: u64,
		/// time since last power on (relative time in nanoseconds)
		uptime: u64,
		/// last power off period (5 second accuracy, in nanoseconds)
		downtime: u64,
	},

	/// Acknowledgement - 45
	///
	/// Response to any message sent with ack_required set to 1. See message header frame address.
	///
	/// (Note that technically this message has no payload, but the frame sequence number is stored
	/// here for convenience).
	Acknowledgement { seq: u8 },

	/// GetLocation - 48
	///
	/// Ask the bulb to return its location information. No payload is required. Causes the device
	/// to transmit a [Message::StateLocation] message.
	GetLocation,

	/// SetLocation -- 49
	///
	/// Set the device location
	SetLocation {
		/// GUID byte array
		location: LifxIdent,
		/// text label for location
		label: LifxString,
		/// UTC timestamp of last label update in nanoseconds
		updated_at: u64,
	},

	/// StateLocation - 50
	///
	/// Device location.
	StateLocation {
		location: LifxIdent,
		label: LifxString,
		updated_at: u64,
	},

	/// GetGroup - 51
	///
	/// Ask the bulb to return its group membership information.
	/// No payload is required.
	/// Causes the device to transmit a [Message::StateGroup] message.
	GetGroup,

	/// SetGroup - 52
	///
	/// Set the device group
	SetGroup {
		group: LifxIdent,
		label: LifxString,
		updated_at: u64,
	},

	/// StateGroup - 53
	///
	/// Device group.
	StateGroup {
		group: LifxIdent,
		label: LifxString,
		updated_at: u64,
	},

	/// EchoRequest - 58
	///
	/// Request an arbitrary payload be echoed back. Causes the device to transmit an [Message::EchoResponse]
	/// message.
	EchoRequest { payload: EchoPayload },

	/// EchoResponse - 59
	///
	/// Response to [Message::EchoRequest] message.
	///
	/// Echo response with payload sent in the EchoRequest.
	///
	EchoResponse { payload: EchoPayload },

	/// Get - 101
	///
	/// Sent by a client to obtain the light state. No payload required. Causes the device to
	/// transmit a [Message::LightState] message.
	LightGet,

	/// SetColor - 102
	///
	/// Sent by a client to change the light state.
	///
	/// If the Frame Address res_required field is set to one (1) then the device will transmit a
	/// State message.
	LightSetColor {
		reserved: u8,
		/// Color in HSBK
		color: HSBK,
		/// Color transition time in milliseconds
		duration: u32,
	},

	/// SetWaveform - 103
	///
	/// Apply an effect to the bulb.
	SetWaveform {
		reserved: u8,
		transient: bool,
		color: HSBK,
		/// Duration of a cycle in milliseconds
		period: u32,
		/// Number of cycles
		cycles: f32,
		/// Waveform Skew, [-32768, 32767] scaled to [0, 1].
		skew_ratio: i16,
		/// Waveform to use for transition.
		waveform: Waveform,
	},

	/// State - 107
	///
	/// Sent by a device to provide the current light state.
	LightState {
		color: HSBK,
		reserved: i16,
		power: PowerLevel,
		label: LifxString,
		reserved2: u64,
	},

	/// GetPower - 116
	///
	/// Sent by a client to obtain the power level. No payload required. Causes the device to
	/// transmit a StatePower message.
	LightGetPower,

	/// SetPower - 117
	///
	/// Sent by a client to change the light power level.
	///
	/// Field   Type
	/// level   unsigned 16-bit integer
	/// duration    unsigned 32-bit integer
	/// The power level must be either 0 or 65535.
	///
	/// The duration is the power level transition time in milliseconds.
	///
	/// If the Frame Address res_required field is set to one (1) then the device will transmit a
	/// StatePower message.
	LightSetPower { level: u16, duration: u32 },

	/// StatePower - 118
	///
	/// Sent by a device to provide the current power level.
	///
	/// Field   Type
	/// level   unsigned 16-bit integer
	LightStatePower { level: u16 },

	/// SetWaveformOptional - 119
	///
	/// Apply an effect to the bulb.
	SetWaveformOptional {
		reserved: u8,
		transient: bool,
		color: HSBK,
		/// Duration of a cycle in milliseconds
		period: u32,
		/// Number of cycles
		cycles: f32,

		skew_ratio: i16,
		waveform: Waveform,
		set_hue: bool,
		set_saturation: bool,
		set_brightness: bool,
		set_kelvin: bool,
	},

	/// GetInfrared - 120
	///
	/// Gets the current maximum power level of the Infraed channel
	LightGetInfrared,

	/// StateInfrared - 121
	///
	/// Indicates the current maximum setting for the infrared channel.
	LightStateInfrared { brightness: u16 },

	/// SetInfrared -- 122
	///
	/// Set the current maximum brightness for the infrared channel.
	LightSetInfrared { brightness: u16 },

	/// SetColorZones - 501
	///
	/// This message is used for changing the color of either a single or multiple zones.
	/// The changes are stored in a buffer and are only applied once a message with either
	/// [ApplicationRequest::Apply] or [ApplicationRequest::ApplyOnly] set.
	SetColorZones {
		start_index: u8,
		end_index: u8,
		color: HSBK,
		duration: u32,
		apply: ApplicationRequest,
	},

	/// GetColorZones - 502
	///
	/// GetColorZones is used to request the zone colors for a range of zones. The bulb will respond
	/// with either [Message::StateZone] or [Message::StateMultiZone] messages as required to cover
	/// the requested range. The bulb may send state messages that cover more than the requested
	/// zones. Any zones outside the requested indexes will still contain valid values at the time
	/// the message was sent.
	GetColorZones { start_index: u8, end_index: u8 },

	/// StateZone - 503

	/// The StateZone message represents the state of a single zone with the `index` field indicating
	/// which zone is represented. The `count` field contains the count of the total number of zones
	/// available on the device.
	StateZone { count: u8, index: u8, color: HSBK },

	/// StateMultiZone - 506
	///
	/// The StateMultiZone message represents the state of eight consecutive zones in a single message.
	/// As in the StateZone message the `count` field represents the count of the total number of
	/// zones available on the device. In this message the `index` field represents the index of
	/// `color0` and the rest of the colors are the consecutive zones thus the index of the
	/// `color_n` zone will be `index + n`.
	StateMultiZone {
		count: u8,
		index: u8,
		color0: HSBK,
		color1: HSBK,
		color2: HSBK,
		color3: HSBK,
		color4: HSBK,
		color5: HSBK,
		color6: HSBK,
		color7: HSBK,
	},
}

impl Message {
	pub fn get_num(&self) -> u16 {
		match *self {
			Message::GetService => 2,
			Message::StateService { .. } => 3,
			Message::GetHostInfo => 12,
			Message::StateHostInfo { .. } => 13,
			Message::GetHostFirmware => 14,
			Message::StateHostFirmware { .. } => 15,
			Message::GetWifiInfo => 16,
			Message::StateWifiInfo { .. } => 17,
			Message::GetWifiFirmware => 18,
			Message::StateWifiFirmware { .. } => 19,
			Message::GetPower => 20,
			Message::SetPower { .. } => 21,
			Message::StatePower { .. } => 22,
			Message::GetLabel => 23,
			Message::SetLabel { .. } => 24,
			Message::StateLabel { .. } => 25,
			Message::GetVersion => 32,
			Message::StateVersion { .. } => 33,
			Message::GetInfo => 34,
			Message::StateInfo { .. } => 35,
			Message::Acknowledgement { .. } => 45,
			Message::GetLocation => 48,
			Message::SetLocation { .. } => 49,
			Message::StateLocation { .. } => 50,
			Message::GetGroup => 51,
			Message::SetGroup { .. } => 52,
			Message::StateGroup { .. } => 53,
			Message::EchoRequest { .. } => 58,
			Message::EchoResponse { .. } => 59,
			Message::LightGet => 101,
			Message::LightSetColor { .. } => 102,
			Message::SetWaveform { .. } => 103,
			Message::LightState { .. } => 107,
			Message::LightGetPower => 116,
			Message::LightSetPower { .. } => 117,
			Message::LightStatePower { .. } => 118,
			Message::SetWaveformOptional { .. } => 119,
			Message::LightGetInfrared => 120,
			Message::LightStateInfrared { .. } => 121,
			Message::LightSetInfrared { .. } => 122,
			Message::SetColorZones { .. } => 501,
			Message::GetColorZones { .. } => 502,
			Message::StateZone { .. } => 503,
			Message::StateMultiZone { .. } => 506,
		}
	}

	/// Tries to parse the payload in a [RawMessage], based on its message type.
	pub fn from_raw(msg: &RawMessage) -> Result<Message, Error> {
		match msg.protocol_header.typ {
			2 => Ok(Message::GetService),
			3 => Ok(unpack!(msg, StateService, service: u8, port: u32)),
			12 => Ok(Message::GetHostInfo),
			13 => Ok(unpack!(
				msg,
				StateHostInfo,
				signal: f32,
				tx: u32,
				rx: u32,
				reserved: i16
			)),
			14 => Ok(Message::GetHostFirmware),
			15 => Ok(unpack!(
				msg,
				StateHostFirmware,
				build: u64,
				reserved: u64,
				version: u32
			)),
			16 => Ok(Message::GetWifiInfo),
			17 => Ok(unpack!(
				msg,
				StateWifiInfo,
				signal: f32,
				tx: u32,
				rx: u32,
				reserved: i16
			)),
			18 => Ok(Message::GetWifiFirmware),
			19 => Ok(unpack!(
				msg,
				StateWifiFirmware,
				build: u64,
				reserved: u64,
				version: u32
			)),
			20 => Ok(Message::GetPower),
			22 => Ok(unpack!(msg, StatePower, level: u16)),
			23 => Ok(Message::GetLabel),
			25 => Ok(unpack!(msg, StateLabel, label: LifxString)),
			32 => Ok(Message::GetVersion),
			33 => Ok(unpack!(
				msg,
				StateVersion,
				vendor: u32,
				product: u32,
				version: u32
			)),
			35 => Ok(unpack!(
				msg,
				StateInfo,
				time: u64,
				uptime: u64,
				downtime: u64
			)),
			45 => Ok(Message::Acknowledgement {
				seq: msg.frame_addr.sequence,
			}),
			48 => Ok(Message::GetLocation),
			50 => Ok(unpack!(
				msg,
				StateLocation,
				location: LifxIdent,
				label: LifxString,
				updated_at: u64
			)),
			51 => Ok(Message::GetGroup),
			53 => Ok(unpack!(
				msg,
				StateGroup,
				group: LifxIdent,
				label: LifxString,
				updated_at: u64
			)),
			58 => Ok(unpack!(msg, EchoRequest, payload: EchoPayload)),
			59 => Ok(unpack!(msg, EchoResponse, payload: EchoPayload)),
			101 => Ok(Message::LightGet),
			102 => Ok(unpack!(
				msg,
				LightSetColor,
				reserved: u8,
				color: HSBK,
				duration: u32
			)),
			107 => Ok(unpack!(
				msg,
				LightState,
				color: HSBK,
				reserved: i16,
				power: u16,
				label: LifxString,
				reserved2: u64
			)),
			116 => Ok(Message::LightGetPower),
			117 => Ok(unpack!(msg, LightSetPower, level: u16, duration: u32)),
			118 => {
				let mut c = Cursor::new(&msg.payload);
				Ok(Message::LightStatePower {
					level: c.read_val()?,
				})
			}
			121 => Ok(unpack!(msg, LightStateInfrared, brightness: u16)),
			501 => Ok(unpack!(
				msg,
				SetColorZones,
				start_index: u8,
				end_index: u8,
				color: HSBK,
				duration: u32,
				apply: u8
			)),
			502 => Ok(unpack!(msg, GetColorZones, start_index: u8, end_index: u8)),
			503 => Ok(unpack!(msg, StateZone, count: u8, index: u8, color: HSBK)),
			506 => Ok(unpack!(
				msg,
				StateMultiZone,
				count: u8,
				index: u8,
				color0: HSBK,
				color1: HSBK,
				color2: HSBK,
				color3: HSBK,
				color4: HSBK,
				color5: HSBK,
				color6: HSBK,
				color7: HSBK
			)),
			_ => Err(Error::UnknownMessageType(msg.protocol_header.typ)),
		}
	}
}
