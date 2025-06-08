use std::net::ToSocketAddrs;

use regex::Regex;

use crate::error::Error;
use crate::generic::config::{ConfigSession, ConfigurationMode};
use crate::generic::connection::{Connection, SSHConnection};
use crate::generic::device::NetworkDevice;

pub type ArubaSSH = ArubaDevice<SSHConnection>;

/// Aruba network device implementation.
pub struct ArubaDevice<C: Connection> {
    connection: C,
    prompt: Regex,
}

// Constants for error messages when executing commands
const INVALID_INPUT: &str = "Invalid input detected at '^' marker.";
const PLATFORM_NOT_APPLICABLE: &str = "Command not applicable for this platform";

impl<C: Connection<ConnectionHandler = C>> NetworkDevice for ArubaDevice<C> {
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
            prompt: Regex::new(r"\(.+\)\s\[.+\]\s(\(config\)\s)?#$").expect("Invalid prompt regex"),
        };

        device.connection.read(&device.prompt)?;
        device.execute("no paging")?;

        Ok(device)
    }

    fn execute(&mut self, command: &str) -> Result<String, Error> {
        let output = self.connection.execute(command, &self.prompt)?;

        if output.contains(INVALID_INPUT) || output.contains(PLATFORM_NOT_APPLICABLE) {
            return Err(Error::CommandExecution(command.to_string()));
        }

        let prefix = format!("{}\n\r", command);
        let output = output.strip_prefix(&prefix).unwrap_or(&output).to_string();

        Ok(output)
    }

    fn enter_config(&mut self) -> Result<Box<dyn ConfigSession + '_>, Error> {
        self.execute("configure terminal")?;

        Ok(Box::new(ConfigurationMode::new(self)))
    }

    fn exit(&mut self) -> Result<(), Error> {
        self.execute("end")?;

        Ok(())
    }

    fn version(&mut self) -> Result<String, Error> {
        self.execute("show version")
    }

    fn logbuffer(&mut self) -> Result<Vec<String>, Error> {
        let output = self.execute("show log network all")?;
        let lines: Vec<String> = output.lines().map(String::from).collect();

        Ok(lines)
    }

    fn ping(&mut self, ip: &str) -> Result<String, Error> {
        let command = format!("ping {}", ip);

        self.execute(&command)
    }

    fn traceroute(&mut self, ip: &str) -> Result<String, Error> {
        let command = format!("traceroute {}", ip);

        self.execute(&command)
    }
}

#[cfg(test)]
mod tests {
    use crate::{create_network_device, Vendor};

    #[test]
    fn test_aruba() -> anyhow::Result<()> {
        env_logger::try_init().ok();

        let addr = format!("{}:22", "10.123.0.15");
        let user = Some("HBSpy");
        let pass = Some(std::env::var("LO_TESTPASS").expect("LO_TESTPASS not set"));

        let mut ssh = create_network_device(Vendor::Aruba, addr, user, pass.as_deref())?;

        let result = ssh.execute("BAD_COMMAND");
        assert!(result.is_err(), "Expected an Err: {:?}", result);

        let result = ssh.version()?;
        assert!(result.contains("Aruba7010"), "{}", result);

        let result = ssh.ping("10.123.0.1")?;
        assert!(result.contains("Success rate is 100 percent"), "{}", result);

        let _result = ssh.logbuffer()?;

        {
            let _config = ssh.enter_config()?;
        }

        let result = ssh.execute("show hostname")?;
        assert!(result.contains("WRD-AC-1"), "{}", result);

        Ok(())
    }
}
