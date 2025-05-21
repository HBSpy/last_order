use std::net::ToSocketAddrs;

use anyhow::Result;
use regex::Regex;

use crate::generic::config::{Configurable, ConfigurationMode};
use crate::generic::connection::{Connection, SSHConnection};

pub type H3cSSH = H3cDevice<SSHConnection>;

pub struct H3cDevice<C: Connection> {
    connection: C,
    prompt_end: Regex,
}

impl<C: Connection<ConnectionHandler = C>> H3cDevice<C> {
    pub fn connect<A: ToSocketAddrs>(
        addr: A,
        username: Option<&str>,
        password: Option<&str>,
    ) -> Result<Self> {
        let mut device = Self {
            connection: C::connect(addr, username, password)?,
            prompt_end: Regex::new(r"<.*>")?,
        };

        device.connection.read(&device.prompt_end)?;

        device.execute("screen-length disable")?;

        Ok(device)
    }

    pub fn execute(&mut self, command: &str) -> Result<String> {
        self.connection.execute(command, &self.prompt_end)
    }

    pub fn version(&mut self) -> Result<String> {
        self.execute("display version")
    }

    pub fn logbuffer(&mut self) -> Result<String> {
        self.execute("display logbuffer")
    }

    pub fn ping(&mut self, ip: &str) -> Result<String> {
        let command = format!("ping {}", ip);

        self.execute(&command)
    }
}

impl<C: Connection> Configurable for H3cDevice<C> {
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

    let addr = format!("{}:22", "10.123.0.24");
    let user = Some("HBSpy");
    let pass = Some(std::env::var("LO_TESTPASS").expect("LO_TESTPASS not set"));

    let mut ssh = H3cSSH::connect(addr, user, pass.as_deref()).unwrap();

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
