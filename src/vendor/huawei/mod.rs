use std::net::ToSocketAddrs;

use anyhow::Result;
use regex::Regex;

use crate::generic::config::{Configurable, ConfigurationMode};
use crate::generic::connection::{Connection, SSHConnection};

pub type HuaweiSSH = HuaweiDevice<SSHConnection>;

pub struct HuaweiDevice<C: Connection> {
    connection: C,
    prompt_end: Regex,
}

impl<C: Connection<ConnectionHandler = C>> HuaweiDevice<C> {
    pub fn connect<A: ToSocketAddrs>(
        addr: A,
        username: Option<&str>,
        password: Option<&str>,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let mut device = Self {
            connection: C::connect(addr, username, password)?,
            prompt_end: Regex::new(r"<.*>")?,
        };

        let _ = device.connection.read(&device.prompt_end);

        device
            .connection
            .execute("screen-length disable", &device.prompt_end)?;

        Ok(device)
    }

    pub fn version(&mut self) -> Result<String> {
        self.connection.execute("display version", &self.prompt_end)
    }

    pub fn ping(&mut self, ip: &str) -> Result<String> {
        let command = format!("ping {}", ip);
        self.connection.execute(&command, &self.prompt_end)
    }
}

impl<C: Connection> Configurable for HuaweiDevice<C> {
    type SessionType = Self;

    fn enter_config(&mut self) -> Result<ConfigurationMode<Self>> {
        self.prompt_end = Regex::new(r"\[.*\]$")?;
        self.execute("system-view")?;

        Ok(ConfigurationMode::enter(self))
    }

    fn execute(&mut self, command: &str) -> Result<String> {
        self.connection.execute(command, &self.prompt_end)
    }

    fn exit(&mut self) -> Result<()> {
        self.prompt_end = Regex::new(r"<.*>")?;
        self.execute("quit")?;

        Ok(())
    }
}

#[test]
fn test() -> anyhow::Result<()> {
    env_logger::try_init().ok();

    let addr = format!("{}:22", "10.123.255.11");
    let user = Some("HBSpy");
    let pass = Some(std::env::var("LO_TESTPASS").expect("LO_TESTPASS not set"));

    let mut ssh = HuaweiSSH::connect(addr, user, pass.as_deref()).unwrap();

    let result = ssh.version()?;
    assert!(result.contains("CE6850-48S4Q-EI"), "{}", result);

    let result = ssh.ping("10.123.11.60")?;
    assert!(result.contains("0.00% packet loss"), "{}", result);

    {
        let mut config = ssh.enter_config()?;
        config.execute("interface 10GE 1/0/1")?;

        let result = config.execute("display this")?;
        assert!(result.contains("description To-ESXi-100-100"), "{}", result);

        config.execute("quit")?;
    }

    Ok(())
}
