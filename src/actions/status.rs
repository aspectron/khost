use crate::imports::*;

#[derive(Describe, Eq, PartialEq, Debug, Clone, Copy)]
#[caption = "Status and logs"]
pub enum Status {
    /// Go back to the previous menu
    #[describe("Back")]
    Back,
    // #[describe("Domain names")]
    // Domains,
    /// The heart of the Kaspa network
    #[describe("Kaspa p2p node")]
    Kaspad,
    /// Node wRPC load balancer
    #[describe("Resolver")]
    Resolver,
    #[describe("Nginx HTTP proxy")]
    Nginx,
}

impl Action<Context> for Status {
    type Error = Error;
    fn run(&self, _ctx: &mut Context) -> Result<()> {
        match self {
            Status::Back => {}
            Status::Kaspad => {}
            Status::Resolver => {}
            Status::Nginx => {}
        }

        Ok(())
    }
}
