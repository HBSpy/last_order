use std::net::ToSocketAddrs;

use encoding_rs::Encoding;
use strum::EnumString;

pub mod error;
pub mod generic;
pub mod vendor;

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
    pub encoding: &'static Encoding,
}

impl<'a> Default for ConnectConfig<'a> {
    fn default() -> Self {
        Self {
            enable_password: None,
            encoding: encoding_rs::UTF_8,
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dev() -> anyhow::Result<()> {
        env_logger::try_init().ok();

        let ssh = (Vendor::H3C, format!("{}:22", "10.123.0.1"));
        // let ssh = (Vendor::Huawei, format!("{}:22", "10.123.255.11"));
        // let ssh = (Vendor::Aruba, format!("{}:22", "10.123.0.15"));

        let user = Some("HBSpy");
        let pass = Some(std::env::var("LO_TESTPASS").expect("LO_TESTPASS not set"));

        let mut ssh = connect(ssh.0, ssh.1, user, pass.as_deref())?;

        let result = ssh.execute("display wlan ap name WRD-South-3 verbose")?;

        dbg!(&result);

        Ok(())
    }
}
