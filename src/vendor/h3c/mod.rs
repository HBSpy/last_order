use super::prelude::*;

pub type H3cSSH = H3cDevice<SSHConnection>;

/// H3C network device implementation.
pub struct H3cDevice<C: Connection> {
    connection: C,
    prompt: Regex,
}

// Constants for error messages when executing commands
const INVALID_INPUT: [&str; 2] = [
    "% Unrecognized command found at '^' position.",
    "% Too many parameters found at '^' position.",
];

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
        _config: ConnectConfig<'_>,
    ) -> Result<Self, Error> {
        let mut device = Self {
            connection: C::connect(addr, username, password, encoding_rs::UTF_8)?,
            prompt: Regex::new(r"[<\[].*[>\]]$").expect("Invalid prompt regex"),
        };

        device.connection.read(&device.prompt)?;
        device.execute("screen-length disable")?;

        Ok(device)
    }

    fn execute(&mut self, command: &str) -> Result<String, Error> {
        let output = self.connection.execute(command, &self.prompt)?;

        if INVALID_INPUT.iter().any(|&msg| output.contains(msg)) {
            return Err(Error::CommandExecution(CommandError::InvalidInput {
                command: command.to_string(),
            }));
        }

        let prefix = format!("{}\r\n", command);
        let output = output.strip_prefix(&prefix).unwrap_or(&output).to_string();

        Ok(output)
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

    fn logbuffer(&mut self) -> Result<Vec<String>, Error> {
        let output = self.execute("display logbuffer")?;

        /*
         - An identifier of percent sign (%) indicates a log with a level equal to or higher than informational.
         - An identifier of asterisk (*) indicates a debugging log or a trace log.
         - An identifier of caret (^) indicates a diagnostic log.
        */
        let lines: Vec<String> = output
            .lines()
            .skip_while(|line| !line.trim().is_empty())
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
        let command = format!("tracert {}", ip);

        self.execute(&command)
    }
}

#[cfg(test)]
mod tests {
    use crate::{connect, ConnectConfig, Vendor};

    #[test]
    fn test_h3c() -> anyhow::Result<()> {
        env_logger::try_init().ok();

        let addr = format!("{}:22", "10.123.0.24");
        let user = Some("HBSpy");
        let pass = Some(std::env::var("LO_TESTPASS").expect("LO_TESTPASS not set"));

        let mut ssh = connect(
            Vendor::H3C,
            addr,
            user,
            pass.as_deref(),
            ConnectConfig::default(),
        )?;

        let result = ssh.execute("BAD_COMMAND");
        assert!(result.is_err(), "Expected an Err: {:?}", result);

        let result = ssh.version()?;
        assert!(result.contains("H3C E528"), "{}", result);

        let result = ssh.ping("10.123.0.1")?;
        assert!(result.contains("5 packet(s) received"), "{}", result);

        let result = ssh.logbuffer()?;
        assert!(
            result.iter().any(|line| line.contains("WRD-24")),
            "Log buffer does not contain expected entry"
        );

        {
            let mut config = ssh.enter_config()?;
            config.execute("interface GigabitEthernet 1/0/8")?;

            let result = config.execute("display this")?;
            assert!(result.contains("to-HMBP"), "{}", result);

            config.execute("quit")?;
        }

        let result = ssh.execute("display environment")?;
        assert!(result.contains("hotspot 1"), "{}", result);

        Ok(())
    }
}
