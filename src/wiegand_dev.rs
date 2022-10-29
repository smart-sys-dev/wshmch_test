use std::{thread, time::Duration};

use serde::Deserialize;

use crate::{utils, wiegand::Wiegand};
use crate::intio::PinLevel;

#[derive(Deserialize)]
pub struct WiegandConfig {
	pin_0: u16,
	pin_1: u16,
	poll_delay_us: u64,
	cutoff_time_ms: u64,
	active_level: PinLevel
}

pub fn test(config: &WiegandConfig) -> Result<(), String> {
	println!("\n[WIEGAND] Test begin..");
	println!("lean the card to reader, in cosole should be its number..");
	let mut wg = Wiegand::new(config.pin_0, config.pin_1).unwrap();
	wg.set_cutoff(Duration::from_millis(config.cutoff_time_ms));
	wg.set_poll_period(Duration::from_micros(config.poll_delay_us));
	wg.set_active_level(config.active_level.as_gpioval());
	let exiter = utils::Exiter::new();
	loop {
		if exiter.check() {
			break;
		}
		match wg.poll() {
			Some(card) => println!("\tCARD READ({}): 0x{:X}", card.order, card.data),
			None => ()
		}
		thread::sleep(Duration::from_micros(config.poll_delay_us));
	}
	Ok(())
}