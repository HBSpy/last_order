pub mod generic;
pub mod vendor;

#[cfg(test)]
mod tests {
    use std::env;

    use crate::generic::config::Configurable;
    use crate::vendor::h3c::H3cSSH;

    #[test]
    fn it_works() -> anyhow::Result<()> {
        env_logger::init();

        let addr = format!("{}:22", "10.123.0.24");
        let user = Some("HBSpy");
        let pass = Some(env::var("LO_TESTPASS").expect("LO_TESTPASS not set"));

        let mut ssh = H3cSSH::connect(addr, user, pass.as_deref()).unwrap();

        let result = ssh.version()?;
        println!("version: {}", result);

        let result = ssh.ping("10.123.11.60")?;
        println!("ping: {}", result);

        {
            let mut config = ssh.enter_config()?;
            config.execute("interface GigabitEthernet 1/0/8")?;
            let result = config.execute("display this")?;
            println!("{}", result);
            config.execute("quit")?;
        }

        Ok(())
    }
}
