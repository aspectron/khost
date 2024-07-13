use crate::imports::*;
use system::System;

pub struct Context {
    pub args: Args,
    pub system: Arc<System>,
    pub username: String,
    pub config: Config,
    pub terminal_width: usize,
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

        let terminal_width = termion::terminal_size()
            .map(|size| size.1 as usize)
            .unwrap_or(80);

        Ok(Context {
            args,
            system,
            username,
            config,
            terminal_width,
        })
    }

    pub fn terminal_width(&self) -> usize {
        // self.terminal_size.map(|(w, _)| w).unwrap_or(80)
        self.terminal_width
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
