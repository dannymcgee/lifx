#![allow(dead_code)]

use std::{net::{SocketAddr, UdpSocket}, thread, time::{Duration, Instant}};
use anyhow::Result;

use crate::{
	self as lifx,
	BuildOptions,
	HSBK,
	Message,
	PowerLevel,
	RawMessage,
	udp::RefreshableData,
};

const HOUR: Duration = Duration::from_secs(60 * 60);

pub struct Bulb {
	pub last_seen: Instant,
	pub source: u32,
	pub target: u64,
	pub addr: SocketAddr,
	pub model: RefreshableData<(u32, u32)>,
	pub location: RefreshableData<String>,
	pub group: RefreshableData<String>,
	pub name: RefreshableData<String>,
	pub host_firmware: RefreshableData<u32>,
	pub wifi_firmware: RefreshableData<u32>,
	pub power_level: RefreshableData<PowerLevel>,
	pub color: Color,
	sock: UdpSocket,
}

#[derive(Debug)]
pub enum Color {
	Unknown,
	Single(RefreshableData<HSBK>),
	Multi(RefreshableData<Vec<Option<HSBK>>>),
}

impl Bulb {
	pub fn new(source: u32, target: u64, sock: UdpSocket, addr: SocketAddr) -> Bulb {
		Bulb {
			last_seen: Instant::now(),
			source,
			target,
			addr,
			model: RefreshableData::empty(HOUR, Message::GetVersion),
			location: RefreshableData::empty(HOUR, Message::GetLocation),
			group: RefreshableData::empty(HOUR, Message::GetGroup),
			name: RefreshableData::empty(HOUR, Message::GetLabel),
			host_firmware: RefreshableData::empty(HOUR, Message::GetHostFirmware),
			wifi_firmware: RefreshableData::empty(HOUR, Message::GetWifiFirmware),
			power_level: RefreshableData::empty(Duration::from_secs(15), Message::GetPower),
			color: Color::Unknown,
			sock,
		}
	}

	pub fn update(&mut self, addr: SocketAddr) {
		self.last_seen = Instant::now();
		self.addr = addr;
	}

	pub fn query_for_missing_info(&self, sock: &UdpSocket) -> Result<()> {
		self.refresh_if_needed(sock, &self.name)?;
		self.refresh_if_needed(sock, &self.group)?;
		self.refresh_if_needed(sock, &self.model)?;
		self.refresh_if_needed(sock, &self.location)?;
		self.refresh_if_needed(sock, &self.host_firmware)?;
		self.refresh_if_needed(sock, &self.wifi_firmware)?;
		self.refresh_if_needed(sock, &self.power_level)?;
		match &self.color {
			Color::Unknown => (), // we'll need to wait to get info about this bulb's model, so we'll know if it's multizone or not
			Color::Single(d) => self.refresh_if_needed(sock, d)?,
			Color::Multi(d) => self.refresh_if_needed(sock, d)?,
		}

		Ok(())
	}

	pub fn set_color(&self, color: HSBK, duration: Duration) -> Result<()> {
		let options = BuildOptions {
			target: Some(self.target),
			res_required: true,
			source: self.source,
			..Default::default()
		};
		let message = RawMessage::build(&options, Message::LightSetColor {
			color,
			duration: duration.as_millis() as u32,
			reserved: 0,
		})?.pack()?;

		let sock = self.sock.try_clone()?;
		let addr = self.addr;

		thread::spawn(move || {
			sock.send_to(&message, addr).unwrap();
		});

		Ok(())
	}

	fn refresh_if_needed<T>(
		&self,
		sock: &UdpSocket,
		data: &RefreshableData<T>,
	) -> Result<()> {
		if data.needs_refresh() {
			let options = BuildOptions {
				target: Some(self.target),
				res_required: true,
				source: self.source,
				..Default::default()
			};
			let message = RawMessage::build(&options, data.refresh_msg.clone())?;
			sock.send_to(&message.pack()?, self.addr)?;
		}
		Ok(())
	}
}

impl std::fmt::Debug for Bulb {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "{:0>16X}  {:^21}  ", self.target, self.addr)?;

		if let Some(group) = self.group.as_ref() {
			write!(f, "{:<7} / ", group)?;
		}
		if let Some(name) = self.name.as_ref() {
			write!(f, "{:<15}", name)?;
		}
		if let Some((vendor, product)) = self.model.as_ref() {
			if let Some(info) = lifx::get_product_info(*vendor, *product) {
				write!(f, "  {:<11} ", info.name)?;
			} else {
				write!(
					f,
					" - Unknown model (vendor={}, product={}) ",
					vendor, product
				)?;
			}
		}
		if let Some(fw_version) = self.host_firmware.as_ref() {
			write!(f, " McuFW:{:x}", fw_version)?;
		}
		if let Some(fw_version) = self.wifi_firmware.as_ref() {
			write!(f, " WifiFW:{:x}", fw_version)?;
		}
		if let Some(level) = self.power_level.as_ref() {
			if *level == PowerLevel::Enabled {
				write!(f, "  Powered On: ")?;
				match self.color {
					Color::Unknown => write!(f, "??")?,
					Color::Single(ref color) => {
						f.write_str(
							&color
								.as_ref()
								.map(|c| c.describe(false))
								.unwrap_or_else(|| "??".to_owned()),
						)?;
					}
					Color::Multi(ref color) => {
						if let Some(vec) = color.as_ref() {
							write!(f, "Zones: ")?;
							for zone in vec {
								if let Some(color) = zone {
									write!(f, "{} ", color.describe(true))?;
								} else {
									write!(f, "?? ")?;
								}
							}
						}
					}
				}
				// write!(f, ")")?;
			} else {
				write!(f, "  Powered Off")?;
			}
		}
		// write!(f, ")")
		write!(f, "")
	}
}
