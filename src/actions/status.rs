use crate::imports::*;

#[derive(Describe, Eq, PartialEq, Debug, Clone, Copy)]
#[caption = "Manage"]
pub enum Status {
    #[describe("Back")]
    Back,
    #[describe("Kaspa p2p node status")]
    Kaspad,
    #[describe("Resolver status")]
    Resolver,
    #[describe("Nginx status")]
    Nginx,
    #[describe("View service logs")]
    ViewLogs,
    #[describe("Follow service logs")]
    FollowLogs,
}

impl Action for Status {
    fn main(&self, ctx: &mut Context) -> Result<bool> {
        match self {
            Status::Back => Ok(false),
            Status::FollowLogs => {
                log::info("Please note: you will need to restart khost after following logs.\nPress Ctrl+C to stop following logs")?;
                match ctx.select_active_service("Select service to follow logs") {
                    Ok(detail) => {
                        sudo!("journalctl", "-fu", detail.name).inner().run()?;
                    }
                    Err(_) => {
                        println!();
                    }
                }
                Ok(true)
            }
            Status::ViewLogs => {
                match ctx.select_active_service("Select service to view logs") {
                    Ok(detail) => {
                        sudo!("journalctl", "-u", detail.name).inner().run()?;
                    }
                    Err(_) => {
                        println!();
                    }
                }
                Ok(true)
            }
            Status::Kaspad => {
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
            Status::Resolver => {
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
            Status::Nginx => {
                let status = nginx::status()?;
                println!("{}", truncate_to_terminal(status));
                println!();
                Ok(true)
            }
        }
    }
}
