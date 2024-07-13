use crate::imports::*;

#[derive(Describe, Eq, PartialEq, Debug, Clone, Copy)]
#[caption = "Status and logs"]
pub enum Status {
    /// Go back to the previous menu
    #[describe("Back")]
    Back,
    /// The heart of the Kaspa network
    #[describe("Kaspa p2p node")]
    Kaspad,
    /// Node wRPC load balancer
    #[describe("Resolver")]
    Resolver,
    #[describe("Nginx HTTP proxy")]
    Nginx,
}

impl Action for Status {
    fn main(&self, ctx: &mut Context) -> Result<bool> {
        match self {
            Status::Back => Ok(false),
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
                let status = resolver::status()?;
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
