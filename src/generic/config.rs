use anyhow::Result;

pub trait Configurable {
    type SessionType: Configurable;

    fn enter_config(&mut self) -> Result<ConfigurationMode<Self::SessionType>>;
    fn execute(&mut self, command: &str) -> Result<String>;
    fn exit(&mut self) -> Result<()>;
}

pub struct ConfigurationMode<'a, T: Configurable> {
    pub(crate) session: &'a mut T,
}

impl<'a, T: Configurable> ConfigurationMode<'a, T> {
    pub fn enter(session: &mut T) -> ConfigurationMode<T> {
        ConfigurationMode { session }
    }

    pub fn execute(&mut self, command: &str) -> Result<String> {
        self.session.execute(command)
    }
}

impl<T: Configurable> Drop for ConfigurationMode<'_, T> {
    fn drop(&mut self) {
        let _ = self.session.exit();
    }
}
