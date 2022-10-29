use std::thread;
use std::time::Duration;

use serde::Deserialize;
use crate::ccnet;
use crate::utils;
use ccnet::Ccnet;
use ccnet::BaudRate;

#[derive(Deserialize)]
pub struct CcnetDevConfig {
	driver: String,
	addr: u8,
	poll_period_ms: u64,
	baudrate: BaudRate
}

pub fn test(config: &CcnetDevConfig) -> Result<(), String> {
	println!("\n[CCNET] Test begin..");
	let mut cashcode = match Ccnet::new(&config.driver, &config.baudrate) {
		Ok(dev) => dev,
		Err(e) => return Err(format!("Fail to create device: {}", e.to_string()))
	};
	match cashcode.info(config.addr) {
		Ok(info) => println!("Device info: {:?}", info),
		Err(e) => return Err(format!("Fail to get device info: {}", e.to_string()))
	}
	match cashcode.get_bill_table(config.addr) {
		Ok(table) => println!("Bill table: {:?}", table),
		Err(e) => return Err(format!("Fail to get bill table: {}", e.to_string()))
	}
	match cashcode.get_bill_options(config.addr) {
		Ok(bopt) => println!("Bill options: {:?}", bopt),
		Err(e) => return Err(format!("Fail to bill options: {}", e.to_string()))
	}
	let ctl = utils::InController::new(&["y - yes", "n - no"]);
	loop {
		match ctl.try_get() {
			Some(c) => if c == 'q' {break;}
			None => ()
		}
		match cashcode.poll(config.addr) {
			Ok(s) => {
				match s {
					ccnet::Status::Escrow(bill) => {
						println!("Accept bill with code {}?", bill);
						match ctl.get() {
							'y' => {
								match cashcode.stack_bill(config.addr) {
									Ok(()) => println!("Bill stacked"),
									Err(e) => return Err(format!("Fail to stack bill: {}", e.to_string()))
								}
							},
							'n' => {
								match cashcode.stack_bill(config.addr) {
									Ok(()) => println!("Bill returned"),
									Err(e) => return Err(format!("Fail to return bill: {}", e.to_string()))
								}
							},
							_ => println!("Type 'y' or 'n'")
						}
					},
					_ => ()
				}
			},
			Err(e) => return Err(format!("Fail to poll device: {}", e.to_string()))
		}
		thread::sleep(Duration::from_millis(config.poll_period_ms));
	}	
	Ok(())
}