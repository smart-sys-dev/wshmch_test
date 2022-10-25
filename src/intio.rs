use std::thread;
use std::time::Duration;

use gpio::GpioValue;
use serde::Deserialize;
use gpio::sysfs::SysFsGpioInput;
use gpio::sysfs::SysFsGpioOutput;
use gpio::{GpioIn, GpioOut};

use crate::utils;

#[derive(Deserialize)]
pub enum PinLevel {
	High,
	Low
}

impl PinLevel {
	pub fn as_gpioval(&self) -> GpioValue {
		match self {
			Self::High => GpioValue::High,
			Self::Low => GpioValue::Low
		}
	}

	pub fn inverse(&self) -> PinLevel {
		match self {
			Self::High => Self::Low,
			Self::Low => Self::High
		}
	}
}

#[derive(Deserialize)]
pub struct IntioConfig {
	pin_i1: u16,
	pin_o1: u16,
	pin_i2: u16,
	pin_o2: u16,
	active_input: PinLevel,
	active_output: PinLevel
}

fn apply_io(input: &mut SysFsGpioInput, output: &mut SysFsGpioOutput, active_input: &PinLevel, active_output: &PinLevel) {
	if input.read_value().unwrap() == active_input.as_gpioval() {
		output.set_value(active_output.as_gpioval()).unwrap();
	} else {
		output.set_value(active_output.inverse().as_gpioval()).unwrap();
	}
}

pub fn test(config: &IntioConfig) {
	println!("\n[INTIO] Test begin..");
	let mut pin_i1 = SysFsGpioInput::open(config.pin_i1).unwrap();
	let mut pin_o1 = SysFsGpioOutput::open(config.pin_o1).unwrap();
	let mut pin_i2 = SysFsGpioInput::open(config.pin_i2).unwrap();
	let mut pin_o2 = SysFsGpioOutput::open(config.pin_o2).unwrap();
	println!("\tPush I1 and O2 should be shorted, I2-O2 too..");
	let exiter = utils::Exiter::new();
	loop {
		apply_io(&mut pin_i1, &mut pin_o1, &config.active_input, &config.active_output);
		apply_io(&mut pin_i2, &mut pin_o2, &config.active_input, &config.active_output);
		if exiter.check() {
			break;
		}
		thread::sleep(Duration::from_millis(10));
	}
}