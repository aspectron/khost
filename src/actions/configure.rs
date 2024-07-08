use crate::imports::*;

#[derive(Describe, Eq, PartialEq, Debug, Clone, Copy)]
pub enum Configure {
    #[describe("Go Back")]
    Back,
    #[describe("Domain names")]
    Domains,
    #[describe("Kaspa p2p node")]
    Kaspad,
    #[describe("Resolver (wRPC load balancer)")]
    Resolver,
}
