use std::net::ToSocketAddrs;

use anyhow::{Context, Result};
use regex::Regex;

use crate::generic::config::{ConfigSession, ConfigurationMode};
use crate::generic::connection::{Connection, SSHConnection};
use crate::generic::device::NetworkDevice;

pub type H3cSSH = H3cDevice<SSHConnection>;

/// H3C network device implementation.
pub struct H3cDevice<C: Connection> {
    connection: C,
    prompt: Regex,
}

impl<C: Connection<ConnectionHandler = C>> NetworkDevice for H3cDevice<C> {
    fn as_any(&mut self) -> &mut dyn std::any::Any
    where
        Self: 'static,
    {
        self
    }

    fn connect<A: ToSocketAddrs>(
        addr: A,
        username: Option<&str>,
        password: Option<&str>,
    ) -> Result<Self> {
        let mut device = Self {
            connection: C::connect(addr, username, password)
                .context("Failed to connect to H3C device")?,
            prompt: Regex::new(r"<.*>$")?,
        };

        device
            .connection
            .read(&device.prompt)
            .context("Failed to read initial prompt")?;
        device
            .execute("screen-length disable")
            .context("Failed to disable screen length")?;

        Ok(device)
    }

    fn execute(&mut self, command: &str) -> Result<String> {
        self.connection
            .execute(command, &self.prompt)
            .context("Failed to execute command")
    }

    fn enter_config(&mut self) -> Result<Box<dyn ConfigSession + '_>> {
        self.prompt = Regex::new(r"\[.*\]$")?;
        self.execute("system-view")
            .context("Failed to enter system-view")?;

        Ok(Box::new(ConfigurationMode::new(self)))
    }

    fn exit(&mut self) -> Result<()> {
        self.prompt = Regex::new(r"<.*>$")?;
        self.execute("quit")
            .context("Failed to exit configuration mode")?;

        Ok(())
    }

    fn version(&mut self) -> Result<String> {
        self.execute("display version")
            .context("Failed to get version")
    }

    fn logbuffer(&mut self) -> Result<String> {
        self.execute("display logbuffer")
            .context("Failed to get log buffer")
    }

    fn ping(&mut self, ip: &str) -> Result<String> {
        let command = format!("ping {}", ip);
        self.execute(&command).context("Ping command failed")
    }
}

#[cfg(test)]
mod tests {
    use crate::vendor::{Vendor, create_network_device};

    #[test]
    fn test_h3c_device() -> anyhow::Result<()> {
        env_logger::try_init().ok();

        let addr = format!("{}:22", "10.123.0.24");
        let user = Some("HBSpy");
        let pass = Some(std::env::var("LO_TESTPASS").expect("LO_TESTPASS not set"));

        let mut ssh = create_network_device(Vendor::H3C, addr, user, pass.as_deref())?;

        let result = ssh.version()?;
        assert!(result.contains("H3C E528"), "{}", result);

        let result = ssh.ping("10.123.11.60")?;
        assert!(result.contains("0.00% packet loss"), "{}", result);

        let result = ssh.logbuffer()?;
        assert!(result.contains("WRD-24"), "{}", result);

        {
            let mut config = ssh.enter_config()?;
            config.execute("interface GigabitEthernet 1/0/8")?;

            let result = config.execute("display this")?;
            assert!(result.contains("to-HPC"), "{}", result);

            config.execute("quit")?;
        }

        Ok(())
    }
}
