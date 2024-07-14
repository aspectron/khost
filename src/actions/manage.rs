use crate::imports::*;

#[derive(Describe, Eq, PartialEq, Debug, Clone, Copy)]
#[caption = "Manage"]
pub enum Manage {
    #[describe("Back")]
    Back,
    #[describe("Enable / Disable services")]
    Enable,
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

impl Action for Manage {
    fn main(&self, ctx: &mut Context) -> Result<bool> {
        match self {
            Manage::Back => Ok(false),
            Manage::Enable => {
                let services = ctx.managed_services();
                let active = ctx.managed_active_services();
                let mut selector =
                    cliclack::multiselect("Select services to enable (ESC to cancel)")
                        .initial_values(active);
                for service in services {
                    selector = selector.item(service.clone(), service, "");
                }
                match selector.interact() {
                    Ok(services) => {
                        enable_services(ctx, services)?;
                    }
                    Err(_) => {
                        println!();
                    }
                }
                Ok(true)
            }
            Manage::FollowLogs => {
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
            Manage::ViewLogs => {
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
            Manage::Kaspad => {
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
            Manage::Resolver => {
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
            Manage::Nginx => {
                let status = nginx::status()?;
                println!("{}", truncate_to_terminal(status));
                println!();
                Ok(true)
            }
        }
    }
}
