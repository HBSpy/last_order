pub mod prelude {
    pub use std::net::ToSocketAddrs;

    pub use regex::Regex;

    pub use crate::error::{CommandError, Error};
    pub use crate::generic::config::{ConfigSession, ConfigurationMode};
    pub use crate::generic::connection::{Connection, SSHConnection};
    pub use crate::generic::device::NetworkDevice;
    pub use crate::ConnectConfig;
}

pub mod aruba;
pub mod cisco;
pub mod h3c;
pub mod huawei;
pub mod ruijie;
