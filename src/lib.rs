use std::net::ToSocketAddrs;

use strum::EnumString;

pub mod generic;
use generic::device::NetworkDevice;

pub mod vendor;

pub mod error;
use error::Error;

#[derive(Debug, Clone, Copy, PartialEq, EnumString)]
#[strum(serialize_all = "snake_case")]
pub enum Vendor {
    Aruba,
    Cisco,
    H3C,
    Huawei,
}

/// Creates a new network device based on the specified vendor.
///
/// # Arguments
/// * `vendor` - The vendor of the device (e.g., Huawei, H3C).
/// * `addr` - The address of the device (e.g., IP and port).
/// * `username` - Optional username for authentication.
/// * `password` - Optional password for authentication.
///
/// # Returns
/// A `Result` containing a `Box<dyn NetworkDevice>` on success, or an error if the connection fails.
pub fn create_network_device<A: ToSocketAddrs>(
    vendor: Vendor,
    addr: A,
    username: Option<&str>,
    password: Option<&str>,
) -> Result<Box<dyn NetworkDevice>, Error> {
    Ok(match vendor {
        Vendor::Aruba => vendor::aruba::ArubaSSH::connect(addr, username, password)?.into_dyn(),
        Vendor::Cisco => vendor::cisco::CiscoSSH::connect(addr, username, password)?.into_dyn(),
        Vendor::H3C => vendor::h3c::H3cSSH::connect(addr, username, password)?.into_dyn(),
        Vendor::Huawei => vendor::huawei::HuaweiSSH::connect(addr, username, password)?.into_dyn(),
    })
}
