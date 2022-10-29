use std::{time::Duration, thread};

use cctalk::{device::CCTalkDevice, protocol::{ChecksumType}};
use serde::Deserialize;
use serial::PortSettings;

use crate::utils;

#[derive(Deserialize)]
pub struct CctalkDevConfig {
	driver: String,
	addr: u8,
	poll_period_ms: u64
}

pub fn test(config: &CctalkDevConfig) -> Result<(), String> {
	println!("\n[CCTALK] Test begin..");
	let port_opt = PortSettings {
		baud_rate: serial::Baud9600,
		char_size: serial::Bits8,
		parity: serial::ParityNone,
		stop_bits: serial::Stop1,
		flow_control: serial::FlowNone
	};
	let mut dev = match CCTalkDevice::new(&config.driver,&port_opt,config.addr,ChecksumType::SimpleChecksum, false) {
		Ok(dev) => dev,
		Err(e) => return Err(format!("Fail to open device: {:?}", e))
	};
	let info = match dev.request_equipment_category() {
		Ok(val) => val,
		Err(e) => return Err(format!("Fail to read info: {:?}", e))
	};
	println!("\tInfo: {}", info);
	println!("\tPut coins to coinacceptor, num of coins should be printed to console..");
	let exiter = utils::Exiter::new();
	loop {
		if exiter.check() {
			break;
		}
		match dev.simple_poll() {
			Ok(data) => {
				println!("Response: {:?}", data);
			},
			Err(e) => return Err(format!("Fail to poll: {:?}", e))
		}
		thread::sleep(Duration::from_millis(config.poll_period_ms));
	}
	Ok(())
}