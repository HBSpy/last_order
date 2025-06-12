use crate::{connect, Vendor};

#[test]
fn test_dev() -> anyhow::Result<()> {
    env_logger::try_init().ok();

    let addr = format!("{}:22", "127.0.0.1");
    let user = Some("username");
    let pass = Some("password");

    let mut ssh = connect(Vendor::H3C, addr, user, pass)?;

    let result = ssh.execute("display version")?;

    dbg!(result);

    Ok(())
}
