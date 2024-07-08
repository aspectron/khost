use crate::imports::*;

mod bootstrap;
pub use bootstrap::*;
mod main;
pub use main::*;
mod advanced;
pub use advanced::*;
mod configure;
pub use configure::*;

#[derive(Describe, Eq, PartialEq, Debug, Clone, Copy)]
pub enum Services {
    #[describe("Kaspa p2p Node")]
    Kaspad,
    #[describe("Resolver (wRPC load balancer)")]
    Resolver,
    #[describe("Nginx (HTTP proxy)")]
    Nginx,
}
