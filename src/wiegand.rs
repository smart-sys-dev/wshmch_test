use std::time::{Duration, Instant};

use gpio::sysfs::SysFsGpioInput;
use gpio::{GpioIn, GpioValue};

const CUTOFF_DEF: Duration = Duration::from_millis(25);
const POLL_DERIOD_DEF: Duration = Duration::from_micros(500);
const ACTIVE_LEVEL_DEF: GpioValue = GpioValue::Low;
const MIN_ORDER_DEF: usize = 24;
const MAX_ORDER_DEF: usize = 44;

pub struct WiegandMsg {
	pub data: u64,
	pub order: usize
}

pub struct Wiegand {
	pin_0: SysFsGpioInput,
	pin_1: SysFsGpioInput,

	cutoff: Duration,
	poll_period: Duration,
	active_level: GpioValue,
	min_order: usize,
	max_order: usize,

	buf: u64,
	pos: usize,
	tl_poll: Instant,
	tl_active: Instant,
	pin_0_release: bool,
	pin_1_release: bool
}

impl Wiegand {
	pub fn new(pin_0: u16, pin_1: u16) -> Result<Self, std::io::Error> {
		Ok(Self {
			pin_0: SysFsGpioInput::open(pin_0)?,
			pin_1: SysFsGpioInput::open(pin_1)?,

			cutoff: CUTOFF_DEF,
			poll_period: POLL_DERIOD_DEF,
			active_level: ACTIVE_LEVEL_DEF,
			min_order: MIN_ORDER_DEF,
			max_order: MAX_ORDER_DEF,

			buf: 0,
			pos: 0,
			tl_poll: Instant::now(),
			tl_active: Instant::now(),
			pin_0_release: false,
			pin_1_release: false
		})
	}

	pub fn set_max_order(&mut self, order: usize) -> Result<(), std::io::Error> {
		if order <= 64 {
			self.max_order = order;
			Ok(())
		} else {
			Err(std::io::Error::new(std::io::ErrorKind::InvalidData, "'order' must be below or equal 64"))
		}
	}

	pub fn set_min_order(&mut self, order: usize) {
		self.min_order = order;
	}

	pub fn set_active_level(&mut self, level: GpioValue) {
		self.active_level = level;
	}

	pub fn set_cutoff(&mut self, dur: Duration) {
		self.cutoff = dur;
	}

	pub fn set_poll_period(&mut self, dur: Duration) {
		self.poll_period = dur;
	}

	pub fn poll(&mut self) -> Option<WiegandMsg> {
		if self.tl_poll.elapsed() >= self.poll_period {
			self.tl_poll = Instant::now();
			if self.pin_0.read_value().unwrap() == self.active_level && self.pin_0_release {
				self.tl_active = Instant::now();
				self.pin_0_release = false;
				if self.pos < self.max_order {
					self.buf = self.buf & !(1 << self.pos);
					self.pos += 1;
				}
				None
			} else {
				self.pin_0_release = true;
				if self.pin_1.read_value().unwrap() == self.active_level {
					self.tl_active = Instant::now();
					self.pin_1_release = false;
					if self.pos < self.max_order {
						self.buf = self.buf | (1 << self.pos);
						self.pos += 1;
					}
					None
				} else {
					self.pin_1_release = true;
					if self.tl_active.elapsed() >= self.cutoff {
						if self.pos >= self.min_order && self.pos <= self.max_order {
							let msg = WiegandMsg {data: self.buf, order: self.pos};
							self.pos = 0;
							self.buf = 0;
							Some(msg)
						} else {
							self.pos = 0;
							self.buf = 0;
							None
						}
					} else {
						None
					}
				}
			}
		} else {
			None
		}
	}
}