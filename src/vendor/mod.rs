use std::net::ToSocketAddrs;

use anyhow::{Context, Result};
use strum::EnumString;

use crate::generic::device::NetworkDevice;

pub mod aruba;
pub mod cisco;
pub mod h3c;
pub mod huawei;

/// Enum to specify the vendor type for the factory function.
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
/// A `Result` containing a `Box<dyn NetworkDevice>` on success, or an error if the connection fails
/// or the vendor is unsupported.
pub fn create_network_device<A: ToSocketAddrs>(
    vendor: Vendor,
    addr: A,
    username: Option<&str>,
    password: Option<&str>,
) -> Result<Box<dyn NetworkDevice>> {
    match vendor {
        Vendor::Huawei => huawei::HuaweiSSH::connect(addr, username, password)
            .map(|device| device.into_dyn())
            .context("Failed to create Huawei device"),
        Vendor::H3C => h3c::H3cSSH::connect(addr, username, password)
            .map(|device| device.into_dyn())
            .context("Failed to create H3C device"),
        Vendor::Cisco => cisco::CiscoSSH::connect(addr, username, password)
            .map(|device| device.into_dyn())
            .context("Failed to create Cisco device"),
        Vendor::Aruba => aruba::ArubaSSH::connect(addr, username, password)
            .map(|device| device.into_dyn())
            .context("Failed to create Aruba device"),
    }
}
