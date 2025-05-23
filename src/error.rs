use std::io;

use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Generic error: {0}")]
    Generic(#[source] io::Error),

    #[error("Authentication failed for user {user}")]
    AuthenticationFailed { user: String },

    #[error("Failed to execute command: {0}")]
    CommandExecution(String),

    #[error("Failed to enter configuration mode")]
    EnterConfigMode,

    #[error("Failed to exit configuration mode")]
    ExitConfigMode,
}
