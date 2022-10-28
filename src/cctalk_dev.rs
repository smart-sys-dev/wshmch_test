use serde::Deserialize;

#[derive(Deserialize)]
pub struct CctalkDevConfig {

}

pub fn test(config: &CctalkDevConfig) -> Result<(), String> {
	Ok(())
}