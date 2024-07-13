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

    pub fn terminal_width(&self) -> usize {
        termion::terminal_size()
            .map(|size| size.0 as usize)
            .unwrap_or(80)
    }

    pub fn truncate<S>(&self, text: S) -> String
    where
        S: Display,
    {
        let text = text.to_string();
        let terminal_width = self.terminal_width();
        text.lines()
            .map(|line| {
                let mut line = line.to_string();
                if line.len() > terminal_width {
                    line.truncate(terminal_width - 3);
                    line.push_str("...");
                }
                line
            })
            .collect::<Vec<_>>()
            .join("\n")
    }
}
