use crate::imports::*;

#[derive(Describe, Eq, PartialEq, Debug, Clone, Copy)]
#[caption = "Status and logs"]
pub enum Status {
    #[describe("Back")]
    Back,
    // #[describe("Service status")]
    // FollowLogs,
    #[describe("Follow logs")]
    FollowLogs,
    #[describe("View logs")]
    ViewLogs,
    #[describe("Kaspa p2p node status")]
    Kaspad,
    #[describe("Resolver service status")]
    Resolver,
    #[describe("Nginx service status")]
    Nginx,
}

impl Action for Status {
    fn main(&self, ctx: &mut Context) -> Result<bool> {
        match self {
            Status::Back => Ok(false),
            Status::FollowLogs => {
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

                Ok(true)
            }
            Status::Resolver => {
                let config = &ctx.config.resolver;
                let status = resolver::status(config)?;
                println!("{}", truncate_to_terminal(status));
                println!();
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
