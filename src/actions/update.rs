use crate::imports::*;

#[derive(Describe, Eq, PartialEq, Debug, Clone, Copy)]
#[caption = "Select services to update"]
pub enum Update {
    /// Go back to the previous menu
    #[describe("Back")]
    Back,
    /// Update OS and all services
    #[describe("Update all")]
    All,
    #[describe("OS prerequisites")]
    Os,
    #[describe("Resolver")]
    Resolver,
    #[describe("Kaspa p2p node")]
    Kaspad,
}

impl Action for Update {
    fn main(&self, ctx: &mut Context) -> Result<()> {
        match self {
            Update::All => {
                base::update(ctx)?;
                resolver::update(ctx)?;
                kaspad::update(ctx)?;
                Ok(())
            }
            Update::Os => {
                base::update(ctx)?;
                Ok(())
            }
            Update::Resolver => {
                resolver::update(ctx)?;
                Ok(())
            }
            Update::Kaspad => {
                kaspad::update(ctx)?;
                Ok(())
            }
            Update::Back => Ok(()),
        }
    }
}
