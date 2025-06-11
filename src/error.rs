use std::io;

use thiserror::Error;

#[derive(Error, Debug)]
pub enum CommandError {
    #[error("Failed to execute command '{command}': {message}")]
    Generic { command: String, message: String },

    #[error("Insufficient privilege for command '{command}'")]
    NoPrivilege { command: String },

    #[error("Invalid input for command '{command}'")]
    InvalidInput { command: String },
}

#[derive(Error, Debug)]
pub enum Error {
    #[error("Generic error: {0}")]
    Generic(#[source] io::Error),

    #[error("Authentication failed for user {user}")]
    AuthenticationFailed { user: String },

    #[error("Command execution failed: {0}")]
    CommandExecution(#[source] CommandError),

    #[error("Failed to enter configuration mode")]
    EnterConfigMode,

    #[error("Failed to exit configuration mode")]
    ExitConfigMode,
}
