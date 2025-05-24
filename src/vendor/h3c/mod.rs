use std::net::ToSocketAddrs;

use regex::Regex;

use crate::error::Error;
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
    ) -> Result<Self, Error> {
        let mut device = Self {
            connection: C::connect(addr, username, password)?,
            prompt: Regex::new(r"[<\[].*[>\]]$").expect("Invalid prompt regex"),
        };

        device.connection.read(&device.prompt)?;
        device.execute("screen-length disable")?;

        Ok(device)
    }

    fn execute(&mut self, command: &str) -> Result<String, Error> {
        self.connection
            .execute(command, &self.prompt)
            .map_err(|_| Error::CommandExecution(command.to_string()))
    }

    fn enter_config(&mut self) -> Result<Box<dyn ConfigSession + '_>, Error> {
        self.execute("system-view")?;

        Ok(Box::new(ConfigurationMode::new(self)))
    }

    fn exit(&mut self) -> Result<(), Error> {
        self.execute("quit")?;

        Ok(())
    }

    fn version(&mut self) -> Result<String, Error> {
        self.execute("display version")
    }

    fn logbuffer(&mut self) -> Result<String, Error> {
        self.execute("display logbuffer")
    }

    fn ping(&mut self, ip: &str) -> Result<String, Error> {
        let command = format!("ping {}", ip);

        self.execute(&command)
    }
}

#[cfg(test)]
mod tests {
    use crate::{create_network_device, Vendor};

    #[test]
    fn test_h3c_device() -> anyhow::Result<()> {
        env_logger::try_init().ok();

        let addr = format!("{}:22", "10.123.0.24");
        let user = Some("HBSpy");
        let pass = Some(std::env::var("LO_TESTPASS").expect("LO_TESTPASS not set"));

        let mut ssh = create_network_device(Vendor::H3C, addr, user, pass.as_deref())?;

        let result = ssh.version()?;
        assert!(result.contains("H3C E528"), "{}", result);

        let result = ssh.ping("10.123.0.1")?;
        assert!(result.contains("5 packet(s) received"), "{}", result);

        let result = ssh.logbuffer()?;
        assert!(result.contains("WRD-24"), "{}", result);

        {
            let mut config = ssh.enter_config()?;
            config.execute("interface GigabitEthernet 1/0/8")?;

            let result = config.execute("display this")?;
            assert!(result.contains("to-HPC"), "{}", result);

            config.execute("quit")?;
        }

        let result = ssh.execute("display environment")?;
        assert!(result.contains("hotspot 1"), "{}", result);

        Ok(())
    }
}
