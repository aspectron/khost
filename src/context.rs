use crate::imports::*;
use system::System;

pub struct Context {
    pub args: Args,
    pub system: Arc<System>,
    pub username: String,
    pub config: Config,
}

impl Context {
    pub fn try_new(args: Args) -> Result<Self> {
        let username = whoami::username();
        let system = Arc::new(System::default());

        let config = match Config::load() {
            Ok(config) => config,
            Err(_) => {
                let config = Config::default();
                config.save()?;
                config
            }
        };

        Ok(Context {
            args,
            system,
            username,
            config,
        })
    }
}
