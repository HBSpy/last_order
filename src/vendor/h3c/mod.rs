use std::net::ToSocketAddrs;

use anyhow::Result;
use regex::Regex;

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
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let mut device = H3cDevice {
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
}
