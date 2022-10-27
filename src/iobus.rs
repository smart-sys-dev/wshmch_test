use std::{thread, time::Duration};

use i2cdev::{
	linux::{LinuxI2CBus, LinuxI2CMessage},
	core::I2CMessage,
	core::I2CTransfer
};
use serde::Deserialize;

use crate::utils;
use crate::intio::PinLevel;

const CYCLE_DELAY: Duration = Duration::from_millis(2);
const DELAY_AFTER_TRANSFER: Duration = Duration::from_millis(10);

#[derive(Deserialize, Debug)]
enum Chip {
	PCF8574
}

#[derive(Deserialize)]
struct IoBusDevice {
	chip: Chip,
	addr: u8,
	active_input: PinLevel,
	active_output: PinLevel,
	input_mask: u8,
	output_mask: u8
}

#[derive(Deserialize)]
pub struct IobusConfig {
	driver: String,
	devices: Vec<IoBusDevice>
}

fn init_pcf8574(bus: &mut LinuxI2CBus, addr: u8, input_mask: u8) -> Result<(), String> {
	let mut msgs = [
		LinuxI2CMessage::write(&[input_mask]).with_address(addr as u16)
	];
	match bus.transfer(&mut msgs) {
		Ok(_) => {
			thread::sleep(DELAY_AFTER_TRANSFER);
			Ok(())
		},
		Err(e) => Err(format!("Fail write to dev: {}", e.to_string()))
	}
}

fn init(bus: &mut LinuxI2CBus, dev: &IoBusDevice) -> Result<(), String> {
	match dev.chip {
		Chip::PCF8574 => init_pcf8574(bus, dev.addr, dev.input_mask)
	}
}

fn write_pcf8574(bus: &mut LinuxI2CBus, addr: u8, active_out: &PinLevel, output_mask: u8, input_mask: u8, val: bool) -> Result<(), String> {
	let mut buf = if val {
		match active_out {
			PinLevel::High => output_mask,
			PinLevel::Low => !output_mask
		}
	} else {
		match active_out {
			PinLevel::High => !output_mask,
			PinLevel::Low => output_mask
		}
	};
	buf = buf | input_mask;
	let mut msgs = [
		LinuxI2CMessage::write(&[buf]).with_address(addr as u16)
	];
	match bus.transfer(&mut msgs) {
		Ok(_) => Ok(()),
		Err(e) => Err(format!("Fail read from dev: {}", e.to_string()))
	}
}

fn read_pcf8574(bus: &mut LinuxI2CBus, addr: u8, active_in: &PinLevel, input_mask: u8) -> Result<bool, String> {
	let mut buf: [u8;1] = [0;1];
	let mut msgs = [
		LinuxI2CMessage::read(&mut buf).with_address(addr as u16)
	];
	match bus.transfer(&mut msgs) {
		Ok(_) => {
			thread::sleep(DELAY_AFTER_TRANSFER);
			let but_is_high = (buf[0] & input_mask) > 0;
			match active_in {
				PinLevel::High => Ok(but_is_high),
				PinLevel::Low => Ok(!but_is_high)
			}
		},
		Err(e) => Err(format!("Fail read from dev: {}", e.to_string()))
	}
}

fn write_dev(bus: &mut LinuxI2CBus, dev: &IoBusDevice, val: bool) -> Result<(), String> {
	match dev.chip {
		Chip::PCF8574 => write_pcf8574(bus, dev.addr, &dev.active_output, dev.output_mask, dev.input_mask, val)
	}
}

fn read_dev(bus: &mut LinuxI2CBus, dev: &IoBusDevice) -> Result<bool, String> {
	match dev.chip {
		Chip::PCF8574 => read_pcf8574(bus, dev.addr, &dev.active_input, dev.input_mask)
	}
}

pub fn test(config: &IobusConfig) -> Result<String, String> {
	println!("\n[IOBUS] Test begin..");
	let mut bus = LinuxI2CBus::new(&config.driver).unwrap();
	for dev in &config.devices {
		match init(&mut bus, dev) {
			Err(e) => {
				println!("Fail init device {:?} at addr {}: {}", dev.chip, dev.addr, e);
				return Err(format!("Device {:?} at addr {} not found", dev.chip, dev.addr));
			},
			_ => println!("Init device {:?} at addr {}: OK", dev.chip, dev.addr)
		}
	}
	println!("\tPush button at IO module, all outputs should be shorted when pushed..");
	let exiter = utils::Exiter::new();
	loop {
		for dev in &config.devices {
			match read_dev(&mut bus, dev) {
				Ok(state) => {
					match write_dev(&mut bus, dev, state) {
						Err(e) => return Err(format!("Fail performing dev {:?} at {}: {}", dev.chip, dev.addr, e)),
						_ => ()
					}
				},
				Err(e) => return Err(format!("Fail performing dev {:?} at {}: {}", dev.chip, dev.addr, e))
			}
		}		
		if exiter.check() {
			break;
		}
		thread::sleep(CYCLE_DELAY);
	}

	Ok(format!("OK"))
}