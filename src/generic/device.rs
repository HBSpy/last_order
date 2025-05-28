use std::{any::Any, net::ToSocketAddrs};

use crate::error::Error;

use super::config::ConfigSession;

/// Trait for network devices with vendor-specific behavior.
pub trait NetworkDevice {
    /// Connects to the device with the specified address and credentials.
    fn connect<A: ToSocketAddrs>(
        addr: A,
        username: Option<&str>,
        password: Option<&str>,
    ) -> Result<Self, Error>
    where
        Self: Sized;

    /// Converts the device into a dynamic trait object for storage in a collection.
    fn into_dyn(self) -> Box<dyn NetworkDevice>
    where
        Self: Sized + 'static,
    {
        Box::new(self)
    }

    /// Returns the device as an Any trait object for downcasting.
    fn as_any(&mut self) -> &mut dyn Any
    where
        Self: 'static;

    /// Executes a command on the device and returns the output.
    /// Used for both general commands and commands in configuration mode.
    fn execute(&mut self, command: &str) -> Result<String, Error>;

    fn enter_config(&mut self) -> Result<Box<dyn ConfigSession + '_>, Error>;

    fn exit(&mut self) -> Result<(), Error>;

    /// Retrieves the device version information.
    fn version(&mut self) -> Result<String, Error>;

    /// Retrieves the device log buffer.
    fn logbuffer(&mut self) -> Result<Vec<String>, Error>;

    /// Performs a ping operation to the specified IP.
    fn ping(&mut self, ip: &str) -> Result<String, Error>;
}
