use std::{time::{Duration, Instant}, thread};

use lifx_core::udp::Manager;

#[allow(unreachable_code)]
fn main() -> anyhow::Result<()> {
	let mut mgr = Manager::new()?;

	loop {
		if Instant::now() - mgr.last_discovery > Duration::from_secs(300) {
			mgr.discover()?;
		}
		mgr.refresh();

		println!("\n");
		if let Ok(bulbs) = mgr.bulbs.lock() {
			for bulb in bulbs.values() {
				println!("{:?}", bulb);
				// bulb.set_color(
				// 	HSBK::color(175, 1.0, 0.4),
				// 	Duration::from_millis(5000)
				// )?;
			}
		}

		thread::sleep(Duration::from_secs(5));
	}

	Ok(())
}
