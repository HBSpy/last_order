use crate::error::Error;

use super::device::NetworkDevice;

pub trait ConfigSession {
    fn execute(&mut self, command: &str) -> Result<String, Error>;
}

pub struct ConfigurationMode<'a> {
    pub(crate) session: &'a mut dyn NetworkDevice,
}

impl<'a> ConfigurationMode<'a> {
    pub fn new(session: &'a mut dyn NetworkDevice) -> Self {
        ConfigurationMode { session }
    }
}

impl ConfigSession for ConfigurationMode<'_> {
    fn execute(&mut self, command: &str) -> Result<String, Error> {
        self.session.execute(command)
    }
}

impl Drop for ConfigurationMode<'_> {
    fn drop(&mut self) {
        let _ = self.session.exit();
    }
}
