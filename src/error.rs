use std::io;

use thiserror::Error;

#[derive(Error, Debug)]
pub enum CommandError {
    #[error("'{command}': {message}")]
    Generic { command: String, message: String },

    #[error("'{command}': Insufficient privilege")]
    NoPrivilege { command: String },

    #[error("'{command}': Invalid input")]
    InvalidInput { command: String },
}

#[derive(Error, Debug)]
pub enum Error {
    #[error("Generic error: {0}")]
    Generic(#[source] io::Error),

    #[error("Authentication failed for user {user}")]
    AuthenticationFailed { user: String },

    #[error("Failed to execute command {0}")]
    CommandExecution(#[source] CommandError),

    #[error("Failed to enter configuration mode")]
    EnterConfigMode,

    #[error("Failed to exit configuration mode")]
    ExitConfigMode,

    #[error("Failed to {operation} to {encoding_name}")]
    EncodingError {
        operation: String,
        encoding_name: String,
    },
}
