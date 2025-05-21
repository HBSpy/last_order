use std::net::ToSocketAddrs;

use anyhow::{Context, Result};
use regex::Regex;

use crate::generic::config::{Configurable, ConfigurationMode};
use crate::generic::connection::{Connection, SSHConnection};
use crate::generic::device::NetworkDevice;

pub type ArubaSSH = ArubaDevice<SSHConnection>;

/// Aruba network device implementation.
pub struct ArubaDevice<C: Connection> {
    connection: C,
    prompt: Regex,
}

impl<C: Connection<ConnectionHandler = C>> NetworkDevice for ArubaDevice<C> {
    fn connect<A: ToSocketAddrs>(
        addr: A,
        username: Option<&str>,
        password: Option<&str>,
    ) -> Result<Self> {
        let mut device = Self {
            connection: C::connect(addr, username, password)
                .context("Failed to connect to Aruba device")?,
            prompt: Regex::new(r".*[>#]$")?,
        };

        device
            .connection
            .read(&device.prompt)
            .context("Failed to read initial prompt")?;
        device
            .execute("no paging")
            .context("Failed to disable paging")?;

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
        self.execute("show log all")
            .context("Failed to get log buffer")
    }

    fn ping(&mut self, ip: &str) -> Result<String> {
        let command = format!("ping {}", ip);
        self.execute(&command).context("Ping command failed")
    }
}

impl<C: Connection<ConnectionHandler = C>> Configurable for ArubaDevice<C> {
    type SessionType = Self;

    fn enter_config(&mut self) -> Result<ConfigurationMode<Self>> {
        self.prompt = Regex::new(r".*\(config.*\) #$")?;
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

    #[test]
    fn test_aruba_device() -> anyhow::Result<()> {
        env_logger::try_init().ok();

        let addr = format!("{}:22", "10.123.0.15");
        let user = Some("HBSpy");
        let pass = Some(std::env::var("LO_TESTPASS").expect("LO_TESTPASS not set"));

        let mut ssh = ArubaSSH::connect(addr, user, pass.as_deref())?;

        let result = ssh.version()?;
        assert!(result.contains("Aruba7010"), "{}", result);

        let result = ssh.ping("10.123.11.60")?;
        assert!(result.contains("Success rate is 100 percent"), "{}", result);

        let result = ssh.logbuffer()?;
        assert!(result.contains(""), "{}", result);

        {
            let _config = ssh.enter_config()?;
        }

        Ok(())
    }
}
