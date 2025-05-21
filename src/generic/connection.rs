use std::io::{self, Read, Write};
use std::net::{TcpStream, ToSocketAddrs};
use std::time::Duration;

use anyhow::{Result, anyhow};
use log::debug;
use regex::Regex;
use ssh2::{Channel, MethodType, Session};

pub trait Connection {
    type ConnectionHandler;

    fn connect<A: ToSocketAddrs>(
        addr: A,
        username: Option<&str>,
        password: Option<&str>,
    ) -> Result<Self::ConnectionHandler>;

    fn read(&mut self, prompt_end: &Regex) -> Result<String>;
    fn execute(&mut self, command: &str, prompt_end: &Regex) -> Result<String>;
}

pub struct SSHConnection {
    #[allow(dead_code)]
    sess: Session,
    channel: Channel,
}

impl SSHConnection {
    fn establish_connection<A: ToSocketAddrs>(
        addr: A,
        timeout: Option<Duration>,
    ) -> Result<Session> {
        let tcp = match timeout {
            None => TcpStream::connect(addr)?,
            Some(timeout) => {
                let mut result = None;
                for addr in addr.to_socket_addrs()? {
                    result = Some(TcpStream::connect_timeout(&addr, timeout));
                    match result {
                        Some(Ok(_)) => break,
                        _ => continue,
                    }
                }
                match result {
                    None => Err(anyhow!("No socket address was supplied in addr"))?,
                    Some(result) => result?,
                }
            }
        };
        let mut sess = Session::new()?;
        sess.set_timeout(60000);

        sess.method_pref(MethodType::HostKey, "ssh-rsa")?;

        sess.set_tcp_stream(tcp);
        sess.handshake()?;

        Ok(sess)
    }

    fn make_channel_session(session: Session) -> Result<SSHConnection> {
        let mut channel = session.channel_session()?;
        channel.request_pty("vt100", None, None)?;
        channel.shell()?;

        Ok(SSHConnection {
            sess: session,
            channel,
        })
    }

    pub fn connect_agentauth<A: ToSocketAddrs>(
        addr: A,
        username: &str,
        timeout: Option<Duration>,
    ) -> Result<SSHConnection> {
        let sess = Self::establish_connection(addr, timeout)?;

        sess.userauth_agent(username)?;

        if !sess.authenticated() {
            return Err(anyhow!(
                "Couldn't authenticate properly against SSH Server using SSH Agent."
            ));
        }

        Self::make_channel_session(sess)
    }
}

impl Connection for SSHConnection {
    type ConnectionHandler = SSHConnection;

    fn connect<A: ToSocketAddrs>(
        addr: A,
        username: Option<&str>,
        password: Option<&str>,
    ) -> Result<SSHConnection> {
        if username.is_none() || password.is_none() {
            return Err(anyhow!(
                "Can't connect to SSH without username and password"
            ));
        }

        let sess = Self::establish_connection(addr, None)?;

        let username = username.unwrap();
        let password = password.unwrap();
        sess.userauth_password(username, password)?;

        if !sess.authenticated() {
            return Err(anyhow!(
                "Couldn't authenticate properly against SSH Server using password auth"
            ));
        }

        Self::make_channel_session(sess)
    }

    fn read(&mut self, prompt_end: &Regex) -> Result<String> {
        debug!("Reading...");
        let mut output = String::new();

        loop {
            let mut buf = [0u8; 1024];

            let size = match self.channel.read(&mut buf) {
                Ok(s) => s,
                Err(ref e) if e.kind() == io::ErrorKind::TimedOut => {
                    debug!("Timed out... Assuming no data");
                    break;
                }
                Err(e) => {
                    debug!("Ignored error: {}", e);
                    break;
                }
            };

            let str = String::from_utf8_lossy(&buf[..size]);
            debug!("Read: {}", str);
            output.push_str(&str);

            if prompt_end.is_match(&str) {
                debug!("Found prompt. Ready for next command");
                break;
            }
        }

        Ok(output)
    }

    fn execute(&mut self, command: &str, prompt_end: &Regex) -> Result<String> {
        debug!("Wrote: {}", command);

        self.channel.write_all(command.as_bytes())?;
        self.channel.write_all(b"\n")?;
        self.channel.flush()?;

        self.read(prompt_end)
    }
}
