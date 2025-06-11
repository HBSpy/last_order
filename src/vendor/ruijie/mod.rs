use super::prelude::*;

pub type RuijieSSH = RuijieDevice<SSHConnection>;

/// Ruijie network device implementation.
pub struct RuijieDevice<C: Connection> {
    connection: C,
    prompt: Regex,
    enable_password: Option<String>,
}

impl<C: Connection> RuijieDevice<C> {
    pub fn enable(&mut self) -> Result<(), Error> {
        let command = format!("enable\n{}", self.enable_password.as_deref().unwrap_or(""));

        self.prompt = Regex::new(r"[a-zA-Z0-9_-]+(\(config\))?#$").expect("Invalid prompt regex");
        self.connection.execute(&command, &self.prompt)?;

        Ok(())
    }
}

// Constants for error messages when executing commands
const INVALID_INPUT: &str = "% Invalid input detected at '^' marker.";
const NO_PRIVILEGE: &str = "% User doesn't have sufficient privilege to execute this command.";

impl<C: Connection<ConnectionHandler = C>> NetworkDevice for RuijieDevice<C> {
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
        config: ConnectConfig<'_>,
    ) -> Result<Self, Error> {
        let mut device = Self {
            connection: C::connect(addr, username, password, encoding_rs::GBK)?,
            // prompt: Regex::new(r"[a-zA-Z0-9_-]+(\(config\))?[>#]$").expect("Invalid prompt regex"),
            prompt: Regex::new(r"[<\[].*[>\]]$").expect("Invalid prompt regex"),
            enable_password: config.enable_password.map(String::from),
        };

        device.connection.read(&device.prompt)?;

        match device.execute("terminal length 0") {
            Ok(_) => return Ok(device),
            Err(Error::CommandExecution(CommandError::NoPrivilege { command: _ })) => {
                device.enable()?;
            }
            Err(e) => return Err(e),
        }

        Ok(device)
    }

    fn execute(&mut self, command: &str) -> Result<String, Error> {
        let output = self.connection.execute(command, &self.prompt)?;

        if output.contains(INVALID_INPUT) {
            return Err(Error::CommandExecution(CommandError::InvalidInput {
                command: command.to_string(),
            }));
        }

        if output.contains(NO_PRIVILEGE) {
            return Err(Error::CommandExecution(CommandError::NoPrivilege {
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

        let lines: Vec<String> = output
            .lines()
            .skip_while(|line| !line.starts_with("Log Buffer "))
            .skip(1)
            .map(String::from)
            .collect();

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
    fn test_ruijie() -> anyhow::Result<()> {
        // Placeholder test; update with actual device details
        Ok(())
    }
}
