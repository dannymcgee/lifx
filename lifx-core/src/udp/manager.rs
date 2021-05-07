#![allow(dead_code)]

use std::{
	collections::HashMap,
	net::{IpAddr, SocketAddr, UdpSocket},
	sync::{Arc, Mutex},
	thread,
	time::{Duration, Instant},
};
use anyhow::Result;
use get_if_addrs::{get_if_addrs, IfAddr, Ifv4Addr};

use crate::{
	self as lifx,
	BuildOptions,
	Message,
	RawMessage,
	Service,
	udp::{Bulb, Color, RefreshableData}
};

pub struct Manager {
	pub bulbs: Arc<Mutex<HashMap<u64, Bulb>>>,
	pub last_discovery: Instant,
	pub sock: UdpSocket,
	pub source: u32,
}

impl Manager {
	pub fn new() -> Result<Manager> {
		let sock = UdpSocket::bind("0.0.0.0:56700")?;
		sock.set_broadcast(true)?;

		// spawn a thread that can send to our socket
		let recv_sock = sock.try_clone()?;

		let bulbs = Arc::new(Mutex::new(HashMap::new()));
		let receiver_bulbs = bulbs.clone();
		let source = 0x72757374;

		// spawn a thread that will receive data from our socket and update our internal data structures
		thread::spawn(move || Self::worker(recv_sock, source, receiver_bulbs));

		let mut mgr = Manager {
			bulbs,
			last_discovery: Instant::now(),
			sock,
			source,
		};
		mgr.discover()?;

		Ok(mgr)
	}

	#[allow(clippy::identity_op)]
	fn handle_message(raw: RawMessage, bulb: &mut Bulb) -> Result<(), lifx::Error> {
		match Message::from_raw(&raw)? {
			Message::StateService { port, service } => {
				if port != bulb.addr.port() as u32 || service != Service::UDP {
					println!("Unsupported service: {:?}/{}", service, port);
				}
			}
			Message::StateLabel { label } => bulb.name.update(label.0),
			Message::StateLocation { label, .. } => bulb.location.update(label.0),
			Message::StateVersion {
				vendor, product, ..
			} => {
				bulb.model.update((vendor, product));
				if let Some(info) = lifx::get_product_info(vendor, product) {
					if info.multizone {
						bulb.color = Color::Multi(RefreshableData::empty(
							Duration::from_secs(15),
							Message::GetColorZones {
								start_index: 0,
								end_index: 255,
							},
						))
					} else {
						bulb.color = Color::Single(RefreshableData::empty(
							Duration::from_secs(15),
							Message::LightGet,
						))
					}
				}
			}
			Message::StatePower { level } => bulb.power_level.update(level),
			Message::StateHostFirmware { version, .. } => bulb.host_firmware.update(version),
			Message::StateWifiFirmware { version, .. } => bulb.wifi_firmware.update(version),
			Message::LightState {
				color,
				power,
				label,
				..
			} => {
				if let Color::Single(ref mut d) = bulb.color {
					d.update(color);
					bulb.power_level.update(power);
				}
				bulb.name.update(label.0);
			}
			Message::StateZone {
				count,
				index,
				color,
			} => {
				if let Color::Multi(ref mut d) = bulb.color {
					d.data.get_or_insert_with(|| {
						let mut v = Vec::with_capacity(count as usize);
						v.resize(count as usize, None);
						assert!(index <= count);
						v
					})[index as usize] = Some(color);
				}
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
				if let Color::Multi(ref mut d) = bulb.color {
					let v = d.data.get_or_insert_with(|| {
						let mut v = Vec::with_capacity(count as usize);
						v.resize(count as usize, None);
						assert!(index + 7 <= count);
						v
					});

					v[index as usize + 0] = Some(color0);
					v[index as usize + 1] = Some(color1);
					v[index as usize + 2] = Some(color2);
					v[index as usize + 3] = Some(color3);
					v[index as usize + 4] = Some(color4);
					v[index as usize + 5] = Some(color5);
					v[index as usize + 6] = Some(color6);
					v[index as usize + 7] = Some(color7);
				}
			}
			Message::StateGroup { label, .. } => {
				bulb.group.update(label.to_string())
			}
			unknown => {
				println!("Received, but ignored {:?}", unknown);
			}
		}

		Ok(())
	}

	fn worker(
		recv_sock: UdpSocket,
		source: u32,
		receiver_bulbs: Arc<Mutex<HashMap<u64, Bulb>>>,
	) {
		let mut buf = [0; 1024];
		loop {
			match recv_sock.recv_from(&mut buf) {
				Ok((0, addr)) => println!("Received a zero-byte datagram from {:?}", addr),
				Ok((nbytes, addr)) => match RawMessage::unpack(&buf[0..nbytes]) {
					Ok(raw) => {
						if raw.frame_addr.target == 0 {
							continue;
						}
						if let Ok(mut bulbs) = receiver_bulbs.lock() {
							let sock = recv_sock.try_clone().unwrap();
							let bulb = bulbs
								.entry(raw.frame_addr.target)
								.and_modify(|bulb| bulb.update(addr))
								.or_insert_with(|| {
									Bulb::new(source, raw.frame_addr.target, sock, addr)
								});
							if let Err(e) = Self::handle_message(raw, bulb) {
								println!("Error handling message from {}: {}", addr, e)
							}
						}
					}
					Err(e) => println!("Error unpacking raw message from {}: {}", addr, e),
				},
				Err(e) => panic!("recv_from err {:?}", e),
			}
		}
	}

	#[allow(clippy::single_match)]
	pub fn discover(&mut self) -> Result<()> {
		println!("Doing discovery");

		let opts = BuildOptions {
			source: self.source,
			..Default::default()
		};
		let rawmsg = RawMessage::build(&opts, Message::GetService).unwrap();
		let bytes = rawmsg.pack().unwrap();

		for addr in get_if_addrs().unwrap() {
			match addr.addr {
				IfAddr::V4(Ifv4Addr {
					broadcast: Some(bcast),
					..
				}) => {
					if addr.ip().is_loopback() {
						continue;
					}
					let addr = SocketAddr::new(IpAddr::V4(bcast), 56700);
					println!("Discovering bulbs on LAN {:?}", addr);
					self.sock.send_to(&bytes, &addr)?;
				}
				_ => {}
			}
		}

		self.last_discovery = Instant::now();

		Ok(())
	}

	pub fn refresh(&self) {
		if let Ok(bulbs) = self.bulbs.lock() {
			for bulb in bulbs.values() {
				bulb.query_for_missing_info(&self.sock).unwrap();
			}
		}
	}
}
