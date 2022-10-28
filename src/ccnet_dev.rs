use serde::Deserialize;

#[derive(Deserialize)]
pub struct CcnetDevConfig {

}

pub fn test(config: &CcnetDevConfig) -> Result<(), String> {
	Ok(())
}