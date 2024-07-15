use crate::imports::*;

#[derive(Describe, Eq, PartialEq, Debug, Clone, Copy)]
#[caption = "Configure Branches"]
pub enum Branches {
    #[describe("Back")]
    Back,
    #[describe("Kaspa p2p node status")]
    Kaspad,
    #[describe("Resolver status")]
    Resolver,
}

impl Action for Branches {
    fn main(&self, ctx: &mut Context) -> Result<bool> {
        match self {
            Branches::Back => Ok(false),
            Branches::Kaspad => {
                let active_configs = kaspad::active_configs(ctx).collect::<Vec<_>>();
                if active_configs.is_empty() {
                    log::warning("No active kaspad configurations found")?;
                } else {
                    for config in kaspad::active_configs(ctx) {
                        match kaspad::status(config) {
                            Ok(status) => {
                                println!("{}", truncate_to_terminal(status));
                                println!();
                            }
                            Err(e) => {
                                log::error(format!("Failed to get kaspad status: {}", e))?;
                            }
                        }
                    }
                }

                Ok(true)
            }
            Branches::Resolver => {
                let config = &ctx.config.resolver;
                if !config.enabled() {
                    log::warning("Resolver is not enabled")?;
                } else {
                    let status = resolver::status(config)?;
                    println!("{}", truncate_to_terminal(status));
                    println!();
                }
                Ok(true)
            }
        }
    }
}
