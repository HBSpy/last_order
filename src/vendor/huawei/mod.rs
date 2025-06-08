use std::any::Any;
use std::net::ToSocketAddrs;

use regex::Regex;

use crate::error::Error;
use crate::generic::config::{ConfigSession, ConfigurationMode};
use crate::generic::connection::{Connection, SSHConnection};
use crate::generic::device::NetworkDevice;

pub type HuaweiSSH = HuaweiDevice<SSHConnection>;

/// Huawei network device implementation.
pub struct HuaweiDevice<C: Connection> {
    connection: C,
    prompt: Regex,
}

// Constants for error messages when executing commands
const INVALID_INPUT: &str = "Error: Unrecognized command found at '^' position.";

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
    ) -> Result<Self, Error> {
        let mut device = Self {
            connection: C::connect(addr, username, password)?,
            prompt: Regex::new(r"[<\[].*[>\]]$").expect("Invalid prompt regex"),
        };

        device.connection.read(&device.prompt)?;
        device.execute("screen-length 0 temporary")?;

        Ok(device)
    }

    fn execute(&mut self, command: &str) -> Result<String, Error> {
        let output = self.connection.execute(command, &self.prompt)?;

        if output.contains(INVALID_INPUT) {
            return Err(Error::CommandExecution(command.to_string()));
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
    use crate::{create_network_device, Vendor};

    #[test]
    fn test_huawei() -> anyhow::Result<()> {
        env_logger::try_init().ok();

        let addr = format!("{}:22", "10.123.255.11");
        let user = Some("HBSpy");
        let pass = Some(std::env::var("LO_TESTPASS").expect("LO_TESTPASS not set"));

        let mut ssh = create_network_device(Vendor::Huawei, addr, user, pass.as_deref())?;

        let result = ssh.execute("BAD_COMMAND");
        assert!(result.is_err(), "Expected an Err: {:?}", result);

        let result = ssh.version()?;
        assert!(result.contains("CE6850-48S4Q-EI"), "{}", result);

        let result = ssh.ping("10.123.0.1")?;
        assert!(result.contains("5 packet(s) received"), "{}", result);

        let result = ssh.logbuffer()?;
        assert!(
            result.iter().any(|line| line.contains("WRD-IDC-11")),
            "Log buffer does not contain expected entry"
        );

        {
            let mut config = ssh.enter_config()?;
            config.execute("interface 10GE 1/0/1")?;

            let result = config.execute("display this")?;
            assert!(result.contains("description To-ESXi-100-100"), "{}", result);

            config.execute("quit")?;
        }

        let result = ssh.execute("display device")?;
        assert!(result.contains("FAN1"), "{}", result);

        Ok(())
    }
}
