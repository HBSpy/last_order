# Last Order [WIP]

## Description

A Rust library for managing network devices.

> Like the administrator of the Misaka Network

## Features

- Multiple vendors
- Disable screen-length to avoid pagination
- Raw command execution
- Enter config mode

## TODO

### Vendors

- [x] Aruba
- [x] Cisco
- [x] H3C
- [x] Huawei
- [ ] Ruijie
- [ ] MikroTik

### Commands:

- [x] version
- [x] logbuffer
- [x] ping

## Example

Refer to vendor::\*::tests::test\_\*\_device

## XXX

- Maybe use `russh` for handling massive(Ciphers, Key exchanges, MACs...) SSH connections?
