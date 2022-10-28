use serde::Deserialize;

#[derive(Deserialize)]
pub struct ExtbusConfig {
	driver: String,
	addr: u8
}

pub fn test(config: &ExtbusConfig) -> Result<(), String> {
	
	Ok(())
}