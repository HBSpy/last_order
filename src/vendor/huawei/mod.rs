use std::any::Any;
use std::net::ToSocketAddrs;

use anyhow::{Context, Result};
use regex::Regex;

use crate::generic::config::{ConfigSession, ConfigurationMode};
use crate::generic::connection::{Connection, SSHConnection};
use crate::generic::device::NetworkDevice;

pub type HuaweiSSH = HuaweiDevice<SSHConnection>;

/// Huawei network device implementation.
pub struct HuaweiDevice<C: Connection> {
    connection: C,
    prompt: Regex,
}

impl<C: Connection<ConnectionHandler = C>> NetworkDevice for HuaweiDevice<C> {
    fn as_any(&mut self) -> &mut dyn Any
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
                .context("Failed to connect to Huawei device")?,
            prompt: Regex::new(r"<.*>$")?,
        };

        device
            .connection
            .read(&device.prompt)
            .context("Failed to read initial prompt")?;
        device
            .execute("screen-length 0 temporary")
            .context("Failed to set screen length")?;

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
    use crate::vendor::{create_network_device, Vendor};

    #[test]
    fn test_huawei_device() -> anyhow::Result<()> {
        env_logger::try_init().ok();

        let addr = format!("{}:22", "10.123.255.11");
        let user = Some("HBSpy");
        let pass = Some(std::env::var("LO_TESTPASS").expect("LO_TESTPASS not set"));

        let mut ssh = create_network_device(Vendor::Huawei, addr, user, pass.as_deref())?;

        let result = ssh.version()?;
        assert!(result.contains("CE6850-48S4Q-EI"), "{}", result);

        let result = ssh.ping("10.123.11.60")?;
        assert!(result.contains("0.00% packet loss"), "{}", result);

        let result = ssh.logbuffer()?;
        assert!(result.contains("WRD-IDC-11"), "{}", result);

        {
            let mut config = ssh.enter_config()?;
            config.execute("interface 10GE 1/0/1")?;

            let result = config.execute("display this")?;
            assert!(result.contains("description To-ESXi-100-100"), "{}", result);

            config.execute("quit")?;
        }

        Ok(())
    }
}
