use super::prelude::*;

pub type CiscoSSH = CiscoDevice<SSHConnection>;

/// Cisco network device implementation.
pub struct CiscoDevice<C: Connection> {
    connection: C,
    prompt: Regex,
}

// Constants for error messages when executing commands
const INVALID_INPUT: &str = "% Invalid input detected at '^' marker.";

impl<C: Connection<ConnectionHandler = C>> NetworkDevice for CiscoDevice<C> {
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
        _config: ConnectConfig,
    ) -> Result<Self, Error> {
        let mut device = Self {
            connection: C::connect(addr, username, password, encoding_rs::UTF_8)?,
            prompt: Regex::new(r"[a-zA-Z0-9_-]+(\(config\))?#$").expect("Invalid prompt regex"),
        };

        device.connection.read(&device.prompt)?;
        device.execute("terminal length 0")?;

        Ok(device)
    }

    fn execute(&mut self, command: &str) -> Result<String, Error> {
        let output = self.connection.execute(command, &self.prompt)?;

        if output.contains(INVALID_INPUT) {
            return Err(Error::CommandExecution(CommandError::InvalidInput {
                command: command.to_string(),
            }));
        }

        let prefix = format!("{}\r\n", command);
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
        let output = self.execute("show logging")?;
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
    #[allow(unused_imports)]
    use crate::{connect, Vendor};

    #[ignore = "no test device"]
    #[test]
    fn test_cisco() -> anyhow::Result<()> {
        // Placeholder test; update with actual device details
        Ok(())
    }
}
