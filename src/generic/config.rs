use anyhow::Result;

use super::device::NetworkDevice;

pub trait ConfigSession {
    fn execute(&mut self, command: &str) -> Result<String>;
}

pub struct ConfigurationMode<'a> {
    session: &'a mut dyn NetworkDevice,
}

impl<'a> ConfigurationMode<'a> {
    pub fn new(session: &'a mut dyn NetworkDevice) -> Self {
        ConfigurationMode { session }
    }
}

impl ConfigSession for ConfigurationMode<'_> {
    fn execute(&mut self, command: &str) -> Result<String> {
        self.session.execute(command)
    }
}

impl Drop for ConfigurationMode<'_> {
    fn drop(&mut self) {
        let _ = self.session.exit();
    }
}
