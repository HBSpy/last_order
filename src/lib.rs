pub mod generic;
pub mod vendor;

#[cfg(test)]
mod tests {
    use std::env;

    use crate::vendor::h3c::H3cSSH;

    #[test]
    fn it_works() -> anyhow::Result<()> {
        env_logger::init();

        let username = Some("HBSpy");
        let password = Some(env::var("LO_TESTPASS").expect("LO_TESTPASS not set"));

        let mut ssh = H3cSSH::connect("10.123.0.1:22", username, password.as_deref()).unwrap();

        let result = ssh.version()?;

        println!("{}", result);

        Ok(())
    }
}
