use std::thread;
use std::time::{Duration, Instant};
use std::io::{Error, ErrorKind};

use serde::Deserialize;
use serialport::SerialPort;

const MIN_REQUEST_DELAY: Duration = Duration::from_millis(150);
const READ_TIMEOUT: Duration = Duration::from_millis(2000);
const BREAK_RESET_DUR: Duration =Duration::from_millis(250);

const POLYNOMIAL: u16 = 0x08408;
const SYNC: u8 = 0x02;
const ACK: u8 = 0x00;
const INC_CMD: u8 = 0xFF;
const NACK: u8 = 0xFF;

mod cmd {
	pub const GET_STATUS: u8 = 0x31;
	pub const POLL: u8 = 0x33;
	pub const STACK: u8 = 0x35;
	pub const RETURN: u8 = 0x36;
	pub const EXT_IDENT: u8 = 0x3E;
	pub const GET_BILL_TABLE: u8 = 0x41;
	pub const REC_CASSETTE_STATUS: u8 = 0x70;
}

mod status {
	pub const POWER_UP: u8 = 0x10;
	pub const INITIALIZE: u8 = 0x13;
	pub const IDLING: u8 = 0x14;
	pub const ACCEPTING: u8 = 0x15;
	pub const STACKING: u8 = 0x17;
	pub const RETURNING: u8 = 0x18;
	pub const DISABLED: u8 = 0x19;
	pub const HOLDING: u8 = 0x1A;
	pub const BUSY: u8 = 0x1B;
	pub const REJECTING: u8 = 0x1C;
	pub const DISPENSING: u8 = 0x1D;
	pub const UPLOADING: u8 = 0x1E;
	pub const SETTING_TYPE_CASSETTE: u8 = 0x21;
	pub const DISPENSED: u8 = 0x25;
	pub const UNLOADED: u8 = 0x26;
	pub const INVALID_BILL_NUMBER: u8 = 0x28;
	pub const SET_CASSETTE_TYPE: u8 = 0x29;
	pub const INVALID_COMMAND: u8 = 0x30;
	pub const DROP_CASSETTE_FULL: u8 = 0x41;
	pub const DROP_CASSETTE_REMOVED: u8 = 0x42;
	pub const JAM_IN_ACCEPTOR: u8 = 0x43;
	pub const JAM_IN_STACKER: u8 = 0x44;
	pub const CHEATED: u8 = 0x45;
	pub const ESCROW: u8 = 0x80;
	pub const PACKED: u8 = 0x81;
	pub const RETURNED: u8 = 0x82;
}

#[derive(Deserialize, Debug)]
pub enum BaudRate {
	Slow = 9600,
	Fast = 19200
}

#[derive(Deserialize, Debug)]
pub struct BillOptions {
	pub acceptable: [bool;24],
	pub security: [bool;24],
	pub routing: [bool;24]
}

#[derive(Deserialize, Debug)]
pub struct CassetteStatus {
	pub present: bool,
	pub is_full: bool,
	pub nbills: u8
}

#[derive(Deserialize, Debug)]
pub struct BillDescription {
	pub denomination: f64,
	pub country_code: String
}

#[derive(Deserialize, Debug)]
pub enum Status {
	PowerUp,
	Initialize,
	Idling,
	Disabled,
	Holding,
	Rejecting(String),
	Dispensing,
	DropCasseteFull,
	DropCasseteRemoved,
	Escrow(u8),
	Other(u8)
}

#[derive(Deserialize, Debug)]
pub struct Info {
	part_number: String,
	serial_number: String,
	asset_number: u64,
	boot_version_head: String,
	program_version_head: String,
	boot_version_cpu: String,
	program_version_cpu: String,
	boot_version_packer: String,
	program_version_packer: String,
	boot_version_cassette1: String,
	boot_version_cassette2: String,
	boot_version_cassette3: String,
	program_version_cassette: String
}

#[derive(Deserialize, Debug)]
pub enum Response {
	Ack,
	Nack,
	IncCmd,
	Message(Vec<u8>)
}

pub struct Ccnet {
	port: Box<dyn SerialPort>,
	tl_request: Instant
}

impl Ccnet {
	pub fn new(driver: &str, baudrate: &BaudRate) -> Result<Self, Error> {
		Ok(Self {
			port: {
				serialport::new(driver, *baudrate as u32)
				.timeout(READ_TIMEOUT)
				.parity(serialport::Parity::None)
				.flow_control(serialport::FlowControl::None)
				.open()?
			},
			tl_request: Instant::now()
		})
	}

	pub fn reset_all(&self) -> Result<(), Error> {
		self.port.set_break()?;
		thread::sleep(BREAK_RESET_DUR);
		self.port.clear_break()?;
		Ok(())
	}

	/// 'GET_STATUS' command
	pub fn get_bill_options(&mut self, addr: u8) -> Result<BillOptions, Error> {
		let resp = self.request(addr, cmd::POLL, &[])?;
		match resp {
			Response::Message(data) => {
				if data.len() == 24 {
					Ok(BillOptions {
						acceptable: Self::bits_to_bools(&data[..3], 24)?.try_into().unwrap(),
						security: Self::bits_to_bools(&data[3..6], 24)?.try_into().unwrap(),
						routing: Self::bits_to_bools(&data[6..9], 24)?.try_into().unwrap()
					})
				} else {
					Err(Error::new(ErrorKind::InvalidData, format!("Incorrect data len({}), must be {}", data.len(), 24)))
				}
			},
			Response::Ack => Err(Error::new(ErrorKind::InvalidData, "Device return ACK on POLL request")),
			Response::IncCmd => Err(Error::new(ErrorKind::InvalidData, "INC_CMD")),
			Response::Nack => Err(Error::new(ErrorKind::InvalidData, "NACK"))
		}		
	}

	pub fn poll(&mut self, addr: u8) -> Result<Status, Error> {
		let resp = self.request(addr, cmd::POLL, &[])?;
		match resp {
			Response::Message(data) => {
				if data.len() > 0  {
					match data[0] {
						status::POWER_UP => Ok(Status::PowerUp),
						status::INITIALIZE => Ok(Status::Initialize),
						status::IDLING => Ok(Status::Idling),
						status::DISABLED => Ok(Status::Disabled),
						status::HOLDING => Ok(Status::Holding),
						status::REJECTING => Ok(Status::Rejecting(String::from("Generic rejecting"))),
						status::DISPENSING => Ok(Status::Dispensing),
						status::DROP_CASSETTE_FULL => Ok(Status::DropCasseteFull),
						status::DROP_CASSETTE_REMOVED => Ok(Status::DropCasseteRemoved),
						status::ESCROW => {
							if data.len() > 1 {
								Ok(Status::Escrow(data[1]))
							} else {
								Err(Error::new(ErrorKind::InvalidData, "In 'Escrow' status not specified 'BillType' field"))
							}
						},
						other => Ok(Status::Other(other))
					}
				} else {
					Err(Error::new(ErrorKind::InvalidData, "Data len in message is zero"))
				}
			},
			Response::Ack => Err(Error::new(ErrorKind::InvalidData, "Device return ACK on POLL request")),
			Response::IncCmd => Err(Error::new(ErrorKind::InvalidData, "INC_CMD")),
			Response::Nack => Err(Error::new(ErrorKind::InvalidData, "NACK"))
		}
	}

	/// 'RECYCLING CASSETTE STATUS' command
	pub fn cassette_status(&mut self, addr: u8) -> Result<Vec<CassetteStatus>, Error> {
		let resp = self.request(addr, cmd::REC_CASSETTE_STATUS, &[])?;
		match resp {
			Response::Message(data) => {
				let mut cassettes = Vec::new();
				let ncassettes = data.len()/2;
				for i in 0..ncassettes {
					cassettes.push(CassetteStatus {
						present: data[i*2] & (1 << 7) > 0,
						is_full: data[i*2] & (1 << 6) > 0,
						nbills: data[i*2 + 1]
					})
				}
				Ok(cassettes)
			},
			Response::Ack => Err(Error::new(ErrorKind::InvalidData, "Device return ACK on POLL request")),
			Response::IncCmd => Err(Error::new(ErrorKind::InvalidData, "INC_CMD")),
			Response::Nack => Err(Error::new(ErrorKind::InvalidData, "NACK"))
		}
	}

	pub fn get_bill_table(&mut self, addr: u8) -> Result<Vec<BillDescription>, Error> {
		let resp = self.request(addr, cmd::GET_BILL_TABLE, &[])?;
		match resp {
			Response::Message(data) => {
				let mut bills = Vec::new();
				let nbills = data.len()/5;
				for i in 0..nbills {
					bills.push(BillDescription {
						denomination: {
							let base = data[i*5] as f64;
							let rad = data[i*5+4] & 0b0111_1111;
							if data[i*5+4] & 0b1000_0000 > 0 {
								let div = (10 as i32).pow(rad as u32);
								base / div as f64
							} else {
								let mult = (10 as i32).pow(rad as u32);
								base * mult as f64
							}
						},
						country_code: Self::bin_to_str(&data[i*5+1..i*5+4])
					})
				}
				Ok(bills)
			},
			Response::Ack => Err(Error::new(ErrorKind::InvalidData, "Device return ACK on POLL request")),
			Response::IncCmd => Err(Error::new(ErrorKind::InvalidData, "INC_CMD")),
			Response::Nack => Err(Error::new(ErrorKind::InvalidData, "NACK"))
		}
	}

	/// 'EXTENDED IDENTIFICATION' command
	pub fn info(&mut self, addr: u8) -> Result<Info, Error> {
		let resp = self.request(addr, cmd::EXT_IDENT, &[])?;
		match resp {
			Response::Message(data) => {
				if data.len() == 109 {
					Ok(Info {
						part_number: Self::bin_to_str(&data[0..15]),
						serial_number: Self::bin_to_str(&data[15..27]),
						asset_number: u64::from_be_bytes(data[27..35].try_into().unwrap()),
						boot_version_head: Self::bin_to_str(&data[35..41]),
						program_version_head: Self::bin_to_str(&data[41..61]),
						boot_version_cpu: Self::bin_to_str(&data[61..67]),
						program_version_cpu: Self::bin_to_str(&data[67..73]),
						boot_version_packer: Self::bin_to_str(&data[73..79]),
						program_version_packer: Self::bin_to_str(&data[79..85]),
						boot_version_cassette1: Self::bin_to_str(&data[85..91]),
						boot_version_cassette2: Self::bin_to_str(&data[91..97]),
						boot_version_cassette3: Self::bin_to_str(&data[97..103]),
						program_version_cassette: Self::bin_to_str(&data[103..109])
					})
				} else {
					Err(Error::new(ErrorKind::InvalidData, format!("Incorrect data len({}), must be {}", data.len(), 109)))
				}
			},
			Response::Ack => Err(Error::new(ErrorKind::InvalidData, "Device return ACK on POLL request")),
			Response::IncCmd => Err(Error::new(ErrorKind::InvalidData, "INC_CMD")),
			Response::Nack => Err(Error::new(ErrorKind::InvalidData, "NACK"))
		}
	}

	pub fn stack_bill(&mut self, addr: u8) -> Result<(), Error> {
		match self.request(addr, cmd::STACK, &[])? {
			Response::Ack => Ok(()),
			Response::IncCmd => Err(Error::new(ErrorKind::InvalidData, "INC_CMD")),
			Response::Nack => Err(Error::new(ErrorKind::InvalidData, "NACK")),
			Response::Message(_) => Err(Error::new(ErrorKind::InvalidData, "Device return Message on STACK request"))
		}
	}

	pub fn return_bill(&mut self, addr: u8) -> Result<(), Error> {
		match self.request(addr, cmd::RETURN, &[])? {
			Response::Ack => Ok(()),
			Response::IncCmd => Err(Error::new(ErrorKind::InvalidData, "INC_CMD")),
			Response::Nack => Err(Error::new(ErrorKind::InvalidData, "NACK")),
			Response::Message(_) => Err(Error::new(ErrorKind::InvalidData, "Device return Message on RETURN request"))
		}
	}


	fn request(&mut self, addr: u8, cmd: u8, data: &[u8]) -> Result<Response, Error> {
		let mut buf: [u8;256] = [0;256];
		let len = data.len() + 6;
		if data.len() > 250 {
			return Err(Error::new(ErrorKind::InvalidData, format!("Data too big: {}, max: {}", data.len(), 250)));
		}
		self.request_delay();
		buf[0] = SYNC;
		buf[1] = addr;
		buf[2] = len as u8;
		buf[3] = cmd;
		
		for i in 0..data.len() {
			buf[4+i] = data[i];
		}
		let offset = data.len() + 4;
		let crc = Self::crc16(&buf[..offset]);
		buf[offset] = (crc & 0xFF) as u8;
		buf[offset+1] = ((crc >> 8) & 0xFF) as u8;
		self.port.write_all(&buf[..len])?;
		self.wait_for_transmitting()?;

		match self.receive_response(addr) {
			Ok(resp) => {
				Ok(match resp {
					Response::Message(data) => {
						self.send_ack(addr, true)?;
						Response::Message(data)
					},
					other => other
				})
			},
			Err(e) => {
				self.send_ack(addr, false)?;
				Err(e)
			}
		}
	}

	fn send_ack(&mut self, addr: u8, ack: bool) -> Result<(), Error> {
		let mut buf: [u8;8] = [0;8];
		buf[0] = SYNC;
		buf[1] = addr;
		buf[2] = 6;
		buf[3] = if ack {ACK} else {NACK};
		
		let crc = Self::crc16(&buf[..4]);
		buf[4] = (crc & 0xFF) as u8;
		buf[5] = ((crc >> 8) & 0xFF) as u8;
		self.port.write_all(&buf[..6])?;
		self.wait_for_transmitting()?;
		Ok(())
	}

	fn receive_response(&mut self, from_addr: u8) -> Result<Response, Error> {
		let mut buf: [u8;256] = [0;256];
		let mut char_buf: [u8;1] = [0;1];
		let mut len: usize = 0;
		let mut ptr = 0;
		self.port.clear(serialport::ClearBuffer::Input)?;
		loop {
			self.port.read(&mut char_buf)?;
			// Wait sync byte
			if ptr == 0 {
				if char_buf[0] == SYNC {
					buf[ptr] = char_buf[0];
					ptr += 1;
				}
			} else // Read addr
			if ptr == 1 {
				if char_buf[0] != from_addr {
					return Err(Error::new(ErrorKind::InvalidData, format!("Expected addr in response({}) does nott not match the real address of the response({})", from_addr, char_buf[0])));
				}
				buf[ptr] = char_buf[0];
				ptr += 1;
			} else // Read length of response
			if ptr == 2 {
				len = char_buf[0] as usize;
				if len < 6 {
					return Err(Error::new(ErrorKind::InvalidData, format!("Specified frame len({}) too small", len)));
				}
				buf[ptr] = char_buf[0];
				ptr += 1;
			} else { // Read data
				buf[ptr] = char_buf[0];
				ptr += 1;
				if ptr >= len{
					break;
				}
			}
		}

		// Assert CRC
		let offset = len - 2;
		let crc_real = Self::crc16(&buf[..len]);
		let crc_received = (buf[offset] as u16) + ((buf[offset+1] as u16) << 8);
		if crc_real != crc_received {
			return Err(Error::new(ErrorKind::InvalidData, format!("Integrity error: crc_real: 0x{:X}, received: 0x{:X}", crc_real, crc_received)));
		}

		// Check response type and return data or empty(ACK)
		if len == 6 {
			match buf[3] {
				ACK => Ok(Response::Ack),
				NACK => Ok(Response::Nack),
				INC_CMD => Ok(Response::IncCmd),
				data => Ok(Response::Message(Vec::from([data])))
			}
		} else {
			Ok(Response::Message(buf[3..len-2].to_vec()))
		}
	}

	fn wait_for_transmitting(&mut self) -> Result<(), Error> {
		while self.port.bytes_to_write()? > 0 {
			thread::sleep(Duration::from_millis(2));
		}
		Ok(())
	}

	fn request_delay(&mut self) {
		loop {
			if self.tl_request.elapsed() > MIN_REQUEST_DELAY {
				break;
			}
			thread::sleep(Duration::from_millis(10));
		}
		self.tl_request = Instant::now();
	}

	fn bits_to_bools(src: &[u8], nbits: usize) -> Result<Vec<bool>, Error> {
		let need_bytes = if nbits % 8 == 0 {nbits/8} else {nbits/8 + 1};
		if src.len() < need_bytes {
			return Err(Error::new(ErrorKind::OutOfMemory, format!("Count of bytes from src({}) less then must have({})", src.len(), need_bytes)));
		}
		let mut bools = Vec::new();
		for i in 0..nbits {
			if src[i/8] & (1 << (i % 8)) > 0 {
				bools.push(true);
			} else {
				bools.push(false);
			}
		}

		Ok(bools)
	}

	fn crc16(data: &[u8]) -> u16 {
		let mut crc: u16 = 0;
		for i in 0..data.len() {
			crc ^= data[i] as u16;
			for j in 0..8 {
				crc >>= 1;
				if crc & 0x0001 > 0 {
					crc ^= POLYNOMIAL;
				}
			}
		}
		crc
	}

	fn bin_to_str(bin: &[u8]) -> String {
		match String::from_utf8(bin.to_vec()) {
			Ok(s) => s,
			_ => String::from("UNKNOWN")
		}
	}
}