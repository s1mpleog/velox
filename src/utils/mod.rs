use crate::error::VeloxError;

pub mod send_email;

pub struct Utils {}

impl Utils {
    pub fn load_env(key: &str) -> Result<String, VeloxError> {
        let env = dotenvy::var(key).map_err(|e| VeloxError::EnvError(e.to_string()))?;
        Ok(env)
    }
}
