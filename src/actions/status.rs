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
    fn main(&self, _ctx: &mut Context) -> Result<bool> {
        match self {
            Status::Back => Ok(false),
            Status::Kaspad => Ok(true),
            Status::Resolver => Ok(true),
            Status::Nginx => Ok(true),
        }
    }
}
