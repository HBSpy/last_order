use std::net::ToSocketAddrs;

use anyhow::{Context, Result};
use regex::Regex;

use crate::generic::config::{Configurable, ConfigurationMode};
use crate::generic::connection::{Connection, SSHConnection};
use crate::generic::device::NetworkDevice;

pub type CiscoSSH = CiscoDevice<SSHConnection>;

/// Cisco network device implementation.
pub struct CiscoDevice<C: Connection> {
    connection: C,
    prompt: Regex,
}

impl<C: Connection<ConnectionHandler = C>> NetworkDevice for CiscoDevice<C> {
    fn connect<A: ToSocketAddrs>(
        addr: A,
        username: Option<&str>,
        password: Option<&str>,
    ) -> Result<Self> {
        let mut device = Self {
            connection: C::connect(addr, username, password)
                .context("Failed to connect to Cisco device")?,
            prompt: Regex::new(r".*[>#]$")?,
        };

        device
            .connection
            .read(&device.prompt)
            .context("Failed to read initial prompt")?;
        device
            .execute("terminal length 0")
            .context("Failed to set terminal length")?;

        Ok(device)
    }

    fn execute(&mut self, command: &str) -> Result<String> {
        self.connection
            .execute(command, &self.prompt)
            .context("Failed to execute command")
    }

    fn version(&mut self) -> Result<String> {
        self.execute("show version")
            .context("Failed to get version")
    }

    fn logbuffer(&mut self) -> Result<String> {
        self.execute("show logging")
            .context("Failed to get log buffer")
    }

    fn ping(&mut self, ip: &str) -> Result<String> {
        let command = format!("ping {}", ip);
        self.execute(&command).context("Ping command failed")
    }
}

impl<C: Connection<ConnectionHandler = C>> Configurable for CiscoDevice<C> {
    type SessionType = Self;

    fn enter_config(&mut self) -> Result<ConfigurationMode<Self>> {
        self.prompt = Regex::new(r".*\(config.*\)#$")?;
        self.execute("configure terminal")
            .context("Failed to enter configuration mode")?;

        Ok(ConfigurationMode::new(self))
    }

    fn exit(&mut self) -> Result<()> {
        self.prompt = Regex::new(r".*[>#]$")?;
        self.execute("end")
            .context("Failed to exit configuration mode")?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[ignore = "no test device"]
    #[test]
    fn test_cisco_device() -> anyhow::Result<()> {
        // Placeholder test; update with actual device details
        Ok(())
    }
}
