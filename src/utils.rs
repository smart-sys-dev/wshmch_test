
use std::fs::File;
use std::io::{Read, BufRead};
use std::sync::mpsc::{Receiver, Sender, channel, TryRecvError};
use std::thread;

use serde::Deserialize;

use crate::extbus::ExtbusConfig;
use crate::intio::IntioConfig;
use crate::iobus::IobusConfig;
use crate::ledmatrix::LedmatrixConfig;

#[derive(Deserialize)]
pub struct Config {
	pub extbus: ExtbusConfig,
	pub intio: IntioConfig,
	pub iobus: IobusConfig,
	pub ledmatrix: LedmatrixConfig
}

const CONFIG_PATH: &str = "./config.toml";

pub fn parse_config() -> Config {
	let mut file = File::open(CONFIG_PATH).unwrap();
	let mut buf = Vec::new();
	file.read_to_end(&mut buf).unwrap();
	toml::from_slice(&buf).unwrap()
}


pub struct Exiter {
	rx: Receiver<()>
}

fn exiter_handler(tx: Sender<()>) {
	let mut stdin = std::io::stdin().lock();
	let mut buf = String::new();
	stdin.read_line(&mut buf).unwrap();
	drop(tx);
}

impl Exiter {
	pub fn new() -> Self {
		println!("\ttype 'Enter' for exit");
		let (tx, rx) = channel();
		thread::spawn(|| exiter_handler(tx));
		Self {
			rx: rx
		}
	}

	pub fn check(&self) -> bool {
		match self.rx.try_recv() {
			Ok(()) => false,
			Err(TryRecvError::Empty) => false,
			_ => true
		}
	}
}