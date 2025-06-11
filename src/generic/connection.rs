use std::io::{self, Read, Write};
use std::net::{TcpStream, ToSocketAddrs};
use std::time::Duration;

use encoding_rs::{Encoding, UTF_8};
use log::debug;
use regex::Regex;
use ssh2::{Channel, MethodType, Session};

use crate::error::Error;

/// Trait for establishing and interacting with network connections.
pub trait Connection {
    type ConnectionHandler;

    /// Establishes a connection to the specified address with optional credentials.
    fn connect<A: ToSocketAddrs>(
        addr: A,
        username: Option<&str>,
        password: Option<&str>,
        encoding: &'static Encoding,
    ) -> Result<Self::ConnectionHandler, Error>;

    /// Reads output until a prompt matching the provided regex is found.
    fn read(&mut self, prompt: &Regex) -> Result<String, Error>;

    /// Executes a command and returns the output until the prompt is matched.
    fn execute(&mut self, command: &str, prompt: &Regex) -> Result<String, Error>;
}

/// SSH connection implementation for network devices.
pub struct SSHConnection {
    #[allow(dead_code)]
    sess: Session,
    channel: Channel,
    encoding: &'static Encoding,
}

impl SSHConnection {
    /// Establishes a TCP connection and initializes an SSH session.
    fn establish_connection<A: ToSocketAddrs>(
        addr: A,
        timeout: Option<Duration>,
    ) -> Result<Session, Error> {
        let mut last_error = None;
        let mut tcp = None;

        for addr in addr.to_socket_addrs().map_err(Error::Generic)? {
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
                || {
                    Error::Generic(io::Error::new(
                        io::ErrorKind::Other,
                        "No socket address was supplied in addr",
                    ))
                },
                |e| Error::Generic(e),
            )
        })?;

        let mut sess = Session::new().map_err(|e| Error::Generic(e.into()))?;
        sess.set_timeout(60_000);

        sess.method_pref(MethodType::HostKey, "ssh-rsa")
            .map_err(|e| Error::Generic(e.into()))?;

        sess.set_tcp_stream(tcp);
        sess.handshake().map_err(|e| Error::Generic(e.into()))?;

        Ok(sess)
    }

    /// Creates a new SSH channel session.
    fn make_channel_session(
        session: Session,
        encoding: &'static Encoding,
    ) -> Result<SSHConnection, Error> {
        let mut channel = session
            .channel_session()
            .map_err(|e| Error::Generic(e.into()))?;
        channel
            .request_pty("vt100", None, None)
            .map_err(|e| Error::Generic(e.into()))?;
        channel.shell().map_err(|e| Error::Generic(e.into()))?;

        Ok(SSHConnection {
            sess: session,
            channel,
            encoding,
        })
    }
}

impl Connection for SSHConnection {
    type ConnectionHandler = SSHConnection;

    fn connect<A: ToSocketAddrs>(
        addr: A,
        username: Option<&str>,
        password: Option<&str>,
        encoding: &'static Encoding,
    ) -> Result<SSHConnection, Error> {
        let username = username.unwrap_or("admin");
        let password = password.unwrap_or("admin");

        let sess = Self::establish_connection(addr, None)?;
        sess.userauth_password(username, password)
            .map_err(|_| Error::AuthenticationFailed {
                user: username.to_string(),
            })?;

        if !sess.authenticated() {
            return Err(Error::AuthenticationFailed {
                user: username.to_string(),
            });
        }

        Self::make_channel_session(sess, encoding)
    }

    fn read(&mut self, prompt: &Regex) -> Result<String, Error> {
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
                Err(e) => return Err(Error::Generic(e)),
            }
        }

        Ok(output)
    }

    fn execute(&mut self, command: &str, prompt: &Regex) -> Result<String, Error> {
        debug!("Executing command: {}", command);

        let command_with_newline = format!("{}\n", command);
        let command_bytes = if self.encoding == UTF_8 {
            command_with_newline.as_bytes()
        } else {
            let (cow, _, had_errors) = self.encoding.encode(&command_with_newline);
            if had_errors {
                return Err(Error::Generic(io::Error::new(
                    io::ErrorKind::InvalidData,
                    format!("Failed to encode command to {}", self.encoding.name()),
                )));
            }
            &cow.into_owned()
        };

        self.channel
            .write_all(&command_bytes)
            .and_then(|_| self.channel.flush())
            .map_err(|e| {
                Error::CommandExecution(crate::error::CommandError::Generic {
                    command: command.to_owned(),
                    message: e.to_string(),
                })
            })?;

        let output = self.read(prompt)?;
        let trimmed = prompt.replace_all(&output, "").to_string();

        Ok(trimmed)
    }
}
