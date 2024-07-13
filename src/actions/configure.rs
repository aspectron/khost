use crate::imports::*;

#[derive(Describe, Eq, PartialEq, Debug, Clone, Copy)]
#[caption = "Configure"]
pub enum Configure {
    /// Go back to the previous menu
    #[describe("Back")]
    Back,
    // #[describe("Fail")]
    // Fail,
    // #[describe("Domain names")]
    // Domains,
    // / The heart of the Kaspa network
    // #[describe("Verbose mode")]
    // Verbose,
    /// Enable or disable Kaspa networks
    #[describe("Configure Kaspa networks")]
    Networks,
    /// wRPC load balancer
    #[describe("Resolver (TODO)")]
    Resolver,
}

impl Action for Configure {
    fn main(&self, ctx: &mut Context) -> Result<bool> {
        match self {
            // Configure::Fail => {
            //     cmd!("bash", "fail").run()?;
            // }
            Configure::Back => Ok(false),
            // Configure::Verbose => {
            //     if confirm("Enable verbose mode?")? {
            //         ctx.config.verbose = true;
            //         ctx.config.save()?;
            //     }
            // }
            Configure::Networks => {
                let active = kaspad::active_configs(ctx)
                    .map(|config| config.network())
                    .collect::<Vec<_>>();
                let mut selector =
                    cliclack::multiselect("Select Kaspa networks to enable (ESC to cancel)")
                        .initial_values(active);
                for item in Network::into_iter() {
                    selector = selector.item(item, item, "");
                }
                match selector.interact() {
                    Ok(networks) => {
                        kaspad::configure_networks(ctx, networks)?;
                    }
                    Err(_) => {
                        println!();
                    }
                }
                Ok(true)
            }
            Configure::Resolver => {
                log::info("Resolver configuration not implemented")?;
                Ok(true)
            }
        }
    }
}
