use serde::Deserialize;

#[derive(Deserialize)]
pub struct RfidConfig {

}

pub fn test(config: &RfidConfig) -> Result<(), String> {
	Ok(())
}