use anyhow::Result;

use super::device::NetworkDevice;

/// Trait for devices that support configuration mode.
pub trait Configurable {
    type SessionType: Configurable + NetworkDevice;

    /// Enters configuration mode.
    fn enter_config(&mut self) -> Result<ConfigurationMode<Self::SessionType>>;

    /// Exits the current configuration mode.
    fn exit(&mut self) -> Result<()>;
}

/// Manages configuration mode for a device.
pub struct ConfigurationMode<'a, T: Configurable + NetworkDevice> {
    session: &'a mut T,
}

impl<'a, T: Configurable + NetworkDevice> ConfigurationMode<'a, T> {
    pub fn new(session: &'a mut T) -> Self {
        ConfigurationMode { session }
    }

    /// Executes a command in configuration mode using the device's NetworkDevice implementation.
    pub fn execute(&mut self, command: &str) -> Result<String> {
        self.session.execute(command)
    }
}

impl<T: Configurable + NetworkDevice> Drop for ConfigurationMode<'_, T> {
    fn drop(&mut self) {
        let _ = self.session.exit();
    }
}
