use crate::imports::*;
use system::System;

pub struct Context {
    pub is_root: bool,
    pub system: Rc<System>,
    pub username: String,
    pub config: Config,
}

impl Context {
    pub fn try_new() -> Result<Self> {
        let is_root = is_root();
        let username = whoami::username();
        let system = Rc::new(System::default());

        let config = match Config::load() {
            Ok(config) => config,
            Err(_) => {
                let config = Config::default();
                config.save()?;
                config
            }
        };

        Ok(Context {
            is_root,
            system,
            username,
            config,
        })
    }

    pub fn root(&self) -> Result<()> {
        if !self.is_root {
            return Err(Error::Sudo);
        }
        Ok(())
    }
}
