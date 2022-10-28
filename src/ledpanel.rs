use std::io::Write;
use std::thread;
use std::time::Duration;

use serde::Deserialize;
use spidev::{Spidev, SpidevOptions, SpiModeFlags};
use gpio::GpioValue;
use gpio::sysfs::SysFsGpioOutput;
use gpio::{GpioOut};

use crate::utils;


const CS_NORMAL: GpioValue = GpioValue::High;
const CS_ACTIVE: GpioValue = GpioValue::Low;

#[derive(Deserialize)]
pub struct LedpanelConfig {
	driver: String,
	pin_cs: u16,
	speed: u32,
	led_delay: u32
}

fn show_buf(spi: &mut Spidev, pin_cs: &mut SysFsGpioOutput, buf: &[u8]) {
	pin_cs.set_value(CS_ACTIVE).unwrap();
	spi.write_all(buf).unwrap();
	pin_cs.set_value(CS_NORMAL).unwrap();
}

pub fn test(config: &LedpanelConfig) -> Result<(), String> {
	println!("\n[LEDPANEL] Test begin..");
	let mut spi = match Spidev::open(&config.driver) {
		Ok(spi) => spi,
		Err(e) => return Err(format!("Fail to open driver: {}", e.to_string()))
	};
    let options = SpidevOptions::new()
         .bits_per_word(8)
         .max_speed_hz(config.speed)
		 .mode(SpiModeFlags::SPI_MODE_0 | SpiModeFlags::SPI_3WIRE | SpiModeFlags::SPI_NO_CS)
         .build();
    spi.configure(&options).unwrap();
	let mut pin_cs = SysFsGpioOutput::open(config.pin_cs).unwrap();
	pin_cs.set_value(CS_NORMAL).unwrap();
	println!("Leds on panel should be filling");
	let exiter = utils::Exiter::new();
	let mut buf: [u8;2] = [0;2];
	let mut pos: usize = 0;
	loop {
		if exiter.check() {
			break;
		}
		show_buf(&mut spi, &mut pin_cs, &buf);
		thread::sleep(Duration::from_millis(config.led_delay as u64));
		buf.fill(0);
		for i in 0..pos {
			buf[i/8] |= 1 >> (i % 8);
		}
		pos += 1;
		if pos >= 16 {
			pos = 0;
		}
	}
	Ok(())
}