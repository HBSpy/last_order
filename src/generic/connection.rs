use std::io::{self, Read, Write};
use std::net::{TcpStream, ToSocketAddrs};
use std::time::Duration;

use anyhow::{Context, Result};
use log::debug;
use regex::Regex;
use ssh2::{Channel, MethodType, Session};

/// Trait for establishing and interacting with network connections.
pub trait Connection {
    type ConnectionHandler;

    /// Establishes a connection to the specified address with optional credentials.
    fn connect<A: ToSocketAddrs>(
        addr: A,
        username: Option<&str>,
        password: Option<&str>,
    ) -> Result<Self::ConnectionHandler>;

    /// Reads output until a prompt matching the provided regex is found.
    fn read(&mut self, prompt: &Regex) -> Result<String>;

    /// Executes a command and returns the output until the prompt is matched.
    fn execute(&mut self, command: &str, prompt: &Regex) -> Result<String>;
}

/// SSH connection implementation for network devices.
pub struct SSHConnection {
    #[allow(dead_code)]
    sess: Session,
    channel: Channel,
}

impl SSHConnection {
    /// Establishes a TCP connection and initializes an SSH session.
    fn establish_connection<A: ToSocketAddrs>(
        addr: A,
        timeout: Option<Duration>,
    ) -> Result<Session> {
        let mut last_error = None;
        let mut tcp = None;

        for addr in addr
            .to_socket_addrs()
            .context("Failed to resolve socket address")?
        {
            let result = if let Some(timeout) = timeout {
                TcpStream::connect_timeout(&addr, timeout)
            } else {
                TcpStream::connect(&addr)
            };

            match result {
                Ok(stream) => {
                    tcp = Some(stream);
                    break;
                }
                Err(e) => {
                    last_error = Some(e);
                    continue;
                }
            }
        }

        let tcp = tcp.ok_or_else(|| {
            last_error.map_or_else(
                || anyhow::anyhow!("No socket address was supplied in addr"),
                |e| anyhow::anyhow!("Failed to connect to any address: {}", e),
            )
        })?;

        let mut sess = Session::new().context("Failed to create SSH session")?;
        sess.set_timeout(60_000); // 60 seconds timeout
        sess.method_pref(MethodType::HostKey, "ssh-rsa")
            .context("Failed to set host key preference")?;
        sess.set_tcp_stream(tcp);
        sess.handshake().context("SSH handshake failed")?;

        Ok(sess)
    }

    /// Creates a new SSH channel session.
    fn make_channel_session(session: Session) -> Result<SSHConnection> {
        let mut channel = session
            .channel_session()
            .context("Failed to create channel session")?;
        channel
            .request_pty("vt100", None, None)
            .context("Failed to request PTY")?;
        channel.shell().context("Failed to start shell")?;

        Ok(SSHConnection {
            sess: session,
            channel,
        })
    }

    /// Connects using SSH agent authentication.
    pub fn connect_agentauth<A: ToSocketAddrs>(
        addr: A,
        username: &str,
        timeout: Option<Duration>,
    ) -> Result<SSHConnection> {
        let sess = Self::establish_connection(addr, timeout)?;
        sess.userauth_agent(username)
            .context("SSH agent authentication failed")?;

        if !sess.authenticated() {
            return Err(anyhow::anyhow!(
                "Authentication failed using SSH agent for user: {}",
                username
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
        let username = username.ok_or_else(|| {
            anyhow::anyhow!("Username is required for SSH password authentication")
        })?;
        let password = password.ok_or_else(|| {
            anyhow::anyhow!("Password is required for SSH password authentication")
        })?;

        let sess = Self::establish_connection(addr, None)?;
        sess.userauth_password(username, password)
            .context("Password authentication failed")?;

        if !sess.authenticated() {
            return Err(anyhow::anyhow!(
                "Authentication failed for user: {} using password",
                username
            ));
        }

        Self::make_channel_session(sess)
    }

    fn read(&mut self, prompt: &Regex) -> Result<String> {
        debug!("Reading from SSH channel...");
        let mut output = String::new();
        let mut buf = [0u8; 1024];

        loop {
            match self.channel.read(&mut buf) {
                Ok(0) => {
                    debug!("End of stream reached");
                    break;
                }
                Ok(size) => {
                    let str = String::from_utf8_lossy(&buf[..size]);
                    debug!("Read: {}", str);
                    output.push_str(&str);

                    if prompt.is_match(&str) {
                        debug!("Prompt found, stopping read");
                        break;
                    }
                }
                Err(ref e) if e.kind() == io::ErrorKind::TimedOut => {
                    debug!("Read timeout, assuming no more data");
                    break;
                }
                Err(e) => return Err(anyhow::anyhow!("Read error: {}", e)),
            }
        }

        Ok(output)
    }

    fn execute(&mut self, command: &str, prompt: &Regex) -> Result<String> {
        debug!("Executing command: {}", command);
        self.channel
            .write_all(command.as_bytes())
            .context("Failed to write command")?;
        self.channel
            .write_all(b"\n")
            .context("Failed to write newline")?;
        self.channel.flush().context("Failed to flush channel")?;

        self.read(prompt)
    }
}
