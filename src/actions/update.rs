use crate::imports::*;

#[derive(Describe, Eq, PartialEq, Debug, Clone, Copy)]
#[caption = "Select services to update"]
pub enum Update {
    /// Go back to the previous menu
    #[describe("Back")]
    Back,
    /// Update services
    #[describe("Update services")]
    Services,
    /// Update OS and all services
    #[describe("Update everything")]
    Everything,
    /// Update only OS
    #[describe("OS prerequisites")]
    Os,
    /// Update only Rust Compiler
    #[describe("Rust Compiler")]
    RustC,
    /// Update only Kaspa Resolver
    #[describe("Resolver")]
    Resolver,
    /// Update only Kaspa p2p node
    #[describe("Kaspa p2p node")]
    Kaspad,
}

impl Action for Update {
    fn main(&self, ctx: &mut Context) -> Result<bool> {
        match self {
            Update::Everything => {
                base::update(ctx)?;
                rust::update()?;
                resolver::update(ctx)?;
                kaspad::update(ctx)?;
                Ok(false)
            }
            Update::Services => {
                rust::update()?;
                resolver::update(ctx)?;
                kaspad::update(ctx)?;
                Ok(false)
            }
            Update::Os => {
                base::update(ctx)?;
                Ok(true)
            }
            Update::RustC => {
                rust::update()?;
                Ok(true)
            }
            Update::Resolver => {
                resolver::update(ctx)?;
                Ok(true)
            }
            Update::Kaspad => {
                kaspad::update(ctx)?;
                Ok(true)
            }
            Update::Back => Ok(false),
        }
    }
}
