use std::{convert::TryFrom, io};
use byteorder::{ReadBytesExt, WriteBytesExt};

use crate::{
	error::Error,
	read_write::{LittleEndianReader, LittleEndianWriter},
};

/// Controls how/when multizone devices apply color changes
///
/// See also [Message::SetColorZones].
#[repr(u8)]
#[derive(Debug, Copy, Clone)]
pub enum ApplicationRequest {
	/// Don't apply the requested changes until a message with Apply or ApplyOnly is sent
	NoApply = 0,
	/// Apply the changes immediately and apply any pending changes
	Apply = 1,
	/// Ignore the requested changes in this message and only apply pending changes
	ApplyOnly = 2,
}

impl<T> LittleEndianWriter<ApplicationRequest> for T
where
	T: WriteBytesExt,
{
	fn write_val(&mut self, v: ApplicationRequest) -> Result<(), io::Error> {
		self.write_u8(v as u8)
	}
}

impl TryFrom<u8> for ApplicationRequest {
	type Error = Error;
	fn try_from(val: u8) -> Result<ApplicationRequest, Error> {
		match val {
			0 => Ok(ApplicationRequest::NoApply),
			1 => Ok(ApplicationRequest::Apply),
			2 => Ok(ApplicationRequest::ApplyOnly),
			x => Err(Error::ProtocolError(format!(
				"Unknown application request {}",
				x
			))),
		}
	}
}

#[repr(u8)]
#[derive(Debug, Copy, Clone)]
pub enum Waveform {
	Saw = 0,
	Sine = 1,
	HalfSign = 2,
	Triangle = 3,
	Pulse = 4,
}

impl<T> LittleEndianWriter<Waveform> for T
where
	T: WriteBytesExt,
{
	fn write_val(&mut self, v: Waveform) -> Result<(), io::Error> {
		self.write_u8(v as u8)
	}
}

impl TryFrom<u8> for Waveform {
	type Error = Error;
	fn try_from(val: u8) -> Result<Waveform, Error> {
		match val {
			0 => Ok(Waveform::Saw),
			1 => Ok(Waveform::Sine),
			2 => Ok(Waveform::HalfSign),
			3 => Ok(Waveform::Triangle),
			4 => Ok(Waveform::Pulse),
			x => Err(Error::ProtocolError(format!(
				"Unknown waveform value {}",
				x
			))),
		}
	}
}

/// Bulb color (Hue-Saturation-Brightness-Kelvin)
///
/// # Notes:
///
/// Colors are represented as Hue-Saturation-Brightness-Kelvin, or HSBK
///
/// When a light is displaying whites, saturation will be zero, hue will be ignored, and only
/// brightness and kelvin will matter.
///
/// Normal values for "kelvin" are from 2500 (warm/yellow) to 9000 (cool/blue)
///
/// When a light is displaying colors, kelvin is ignored.
///
/// To display "pure" colors, set saturation to full (65535).
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct HSBK {
	pub hue: u16,
	pub saturation: u16,
	pub brightness: u16,
	pub kelvin: u16,
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Kelvin  {
	Candlelight    = 1500,
	Sunset         = 2000,
	UltraWarm      = 2500,
	Incandescent   = 2700,
	Warm           = 3000,
	Neutral        = 3500,
	Cool           = 4000,
	CoolDaylight   = 4500,
	SoftDaylight   = 5000,
	Daylight       = 5600,
	NoonDaylight   = 6000,
	BrightDaylight = 6500,
	CloudyDaylight = 7000,
	BlueDaylight   = 7500,
	BlueOvercast   = 8000,
	BlueIce        = 9000,
}

impl HSBK {
	pub fn describe(&self, short: bool) -> String {
		match short {
			true if self.saturation == 0 => format!("{}K", self.kelvin),
			true => format!(
				"{:.0}/{:.0}",
				(self.hue as f32 / 65535.0) * 360.0,
				self.saturation as f32 / 655.35
			),
			false if self.saturation == 0 => format!(
				"{:<3.0}% White ({})",
				self.brightness as f32 / 655.35,
				describe_kelvin(self.kelvin)
			),
			false => format!(
				"{:<3.0}% hue: {:<3.0} sat: {:<3.0}%",
				self.brightness as f32 / 655.35,
				(self.hue as f32 / 65535.0) * 360.0,
				self.saturation as f32 / 655.35
			),
		}
	}

	pub fn white(kelvin: u16, brightness: f32) -> HSBK {
		HSBK {
			hue: 0,
			saturation: 0,
			kelvin,
			brightness: (brightness * u16::MAX as f32) as u16,
		}
	}

	pub fn color(hue: u16, saturation: f32, brightness: f32) -> HSBK {
		HSBK {
			hue: ((hue as f32 / 360.0) * (u16::MAX as f32)) as u16,
			saturation: (saturation * u16::MAX as f32) as u16,
			brightness: (brightness * u16::MAX as f32) as u16,
			kelvin: 0,
		}
	}
}

/// Describe (in english words) the color temperature as given in kelvin.
///
/// These descriptions match the values shown in the LIFX mobile app.
pub fn describe_kelvin(k: u16) -> &'static str {
	match k {
		   0..2000 => "Candlelight",
		2000..2500 => "Sunset",
		2500..2700 => "Ulra Warm",
		2700..3000 => "Incandescent",
		3000..3500 => "Warm",
		3500..4000 => "Neutral",
		4000..4500 => "Cool",
		4500..5000 => "Cool Daylight",
		5000..5600 => "Soft Daylight",
		5600..6000 => "Daylight",
		6000..6500 => "Noon Daylight",
		6500..7000 => "Bright Daylight",
		7000..7500 => "Cloudy Daylight",
		7500..8000 => "Blue Daylight",
		8000..9000 => "Blue Overcast",
		9000..=u16::MAX => "Blue Ice",
	}
}

impl HSBK {}

impl<R: ReadBytesExt> LittleEndianReader<HSBK> for R {
	fn read_val(&mut self) -> Result<HSBK, io::Error> {
		let hue = self.read_val()?;
		let sat = self.read_val()?;
		let bri = self.read_val()?;
		let kel = self.read_val()?;
		Ok(HSBK {
			hue,
			saturation: sat,
			brightness: bri,
			kelvin: kel,
		})
	}
}

impl<T> LittleEndianWriter<HSBK> for T
where
	T: WriteBytesExt,
{
	fn write_val(&mut self, v: HSBK) -> Result<(), io::Error> {
		self.write_val(v.hue)?;
		self.write_val(v.saturation)?;
		self.write_val(v.brightness)?;
		self.write_val(v.kelvin)?;
		Ok(())
	}
}
