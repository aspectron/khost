use crate::imports::*;

#[derive(Describe, Eq, PartialEq, Debug, Clone, Copy)]
#[caption = "Configure services"]
pub enum Configure {
    /// Go back to the previous menu
    #[describe("Back")]
    Back,
    // #[describe("Fail")]
    // Fail,
    // #[describe("Domain names")]
    // Domains,
    /// The heart of the Kaspa network
    #[describe("Kaspa p2p node (TODO)")]
    Kaspad,
    /// wRPC load balancer
    #[describe("Resolver (TODO)")]
    Resolver,
}

impl Action for Configure {
    fn main(&self, ctx: &mut Context) -> Result<()> {
        match self {
            // Configure::Fail => {
            //     cmd!("bash", "fail").run()?;
            // }
            Configure::Back => {}
            Configure::Kaspad => {
                let active = kaspad::active_configs(ctx)
                    .map(|config| config.network())
                    .collect::<Vec<_>>();
                let mut selector =
                    cliclack::multiselect("Select Kaspa networks to enable (ESC to cancel)")
                        .initial_values(active);
                for item in Network::into_iter() {
                    selector = selector.item(item, item, "");
                }
                let selection = selector.interact().ok();
                if let Some(networks) = selection {
                    kaspad::configure_networks(ctx, networks)?;
                }
            }
            Configure::Resolver => {
                log::info("Resolver configuration not implemented")?;
            }
        }

        Ok(())
    }
}
