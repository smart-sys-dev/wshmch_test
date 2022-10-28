use serde::Deserialize;

#[derive(Deserialize)]
pub struct TerminalConfig {

}

pub fn test(config: &TerminalConfig) -> Result<(), String> {
	Ok(())
}