# Last Order [WIP]

## Description

A Rust library for managing network devices.

> Like the administrator of the Misaka Network

## Features

- Multiple vendors
- Disable screen-length to avoid pagination
- Raw command execution
- Enter config mode
- Handle GBK encoding for such as Ruijie devices

### Vendors

- [x] Aruba
- [x] Cisco
- [x] H3C
- [x] Huawei
- [x] Ruijie
- [ ] MikroTik

### Commands:

- version
- logbuffer
- ping
- traceroute

## Example

Refer to vendor::\*::tests::test\_\*

## XXX

- Maybe use `russh` for handling massive(Ciphers, Key exchanges, MACs...) SSH connections?
