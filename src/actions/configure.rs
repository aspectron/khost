use crate::imports::*;

#[derive(Describe, Eq, PartialEq, Debug, Clone, Copy)]
#[caption = "Configure services"]
pub enum Configure {
    /// Go back to the previous menu
    #[describe("Back")]
    Back,
    // #[describe("Domain names")]
    // Domains,
    /// The heart of the Kaspa network
    #[describe("Kaspa p2p node (TODO)")]
    Kaspad,
    /// wRPC load balancer
    #[describe("Resolver (TODO)")]
    Resolver,
}

impl Action<Context> for Configure {
    type Error = Error;
    fn run(&self, _ctx: &mut Context) -> Result<()> {
        match self {
            Configure::Back => {}
            Configure::Kaspad => {}
            Configure::Resolver => {}
        }

        Ok(())
    }
}
