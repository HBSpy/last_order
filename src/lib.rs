use std::net::ToSocketAddrs;

use strum::EnumString;

pub mod error;
pub mod generic;
pub mod vendor;

#[cfg(test)]
mod tests;

use generic::device::NetworkDevice;

#[derive(Debug, Clone, Copy, PartialEq, EnumString)]
#[strum(serialize_all = "snake_case")]
pub enum Vendor {
    Aruba,
    Cisco,
    H3C,
    Huawei,
    Ruijie,
}

#[derive(Debug, Clone)]
pub struct ConnectConfig<'a> {
    pub enable_password: Option<&'a str>,
}

impl<'a> Default for ConnectConfig<'a> {
    fn default() -> Self {
        Self {
            enable_password: None,
        }
    }
}

macro_rules! connect_vendor {
    ($($vendor:ident => $module:ident::$type:ident),* $(,)?) => {
        pub fn connect<A: ToSocketAddrs>(
            vendor: Vendor,
            addr: A,
            username: Option<&str>,
            password: Option<&str>,
            config: ConnectConfig,
        ) -> Result<Box<dyn NetworkDevice>, error::Error> {
            Ok(match vendor {
                $(
                    Vendor::$vendor => vendor::$module::$type::connect(addr, username, password, config)?.into_dyn(),
                )*
            })
        }
    };
}

connect_vendor! {
    Aruba => aruba::ArubaSSH,
    Cisco => cisco::CiscoSSH,
    H3C => h3c::H3cSSH,
    Huawei => huawei::HuaweiSSH,
    Ruijie => ruijie::RuijieSSH,
}
