use std::net::ToSocketAddrs;

use anyhow::Result;

/// Trait for network devices with vendor-specific behavior.
pub trait NetworkDevice {
    /// Connects to the device with the specified address and credentials.
    fn connect<A: ToSocketAddrs>(
        addr: A,
        username: Option<&str>,
        password: Option<&str>,
    ) -> Result<Self>
    where
        Self: Sized;

    /// Executes a command on the device and returns the output.
    /// Used for both general commands and commands in configuration mode.
    fn execute(&mut self, command: &str) -> Result<String>;

    /// Retrieves the device version information.
    fn version(&mut self) -> Result<String>;

    /// Retrieves the device log buffer.
    fn logbuffer(&mut self) -> Result<String>;

    /// Performs a ping operation to the specified IP.
    fn ping(&mut self, ip: &str) -> Result<String>;
}
