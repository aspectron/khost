use crate::imports::*;

#[derive(Describe, Eq, PartialEq, Debug, Clone, Copy)]
#[caption = "Select services to update"]
pub enum Update {
    #[describe("Back")]
    Back,
    #[describe("Update Kaspa services")]
    Services,
    #[describe("OS prerequisites")]
    Os,
    #[describe("Rust Compiler")]
    RustC,
    #[describe("Resolver")]
    Resolver,
    #[describe("Kaspa p2p node")]
    Kaspad,
}

impl Action for Update {
    fn main(&self, ctx: &mut Context) -> Result<bool> {
        match self {
            // Update::Everything => {
            //     base::update(ctx)?;
            //     rust::update()?;
            //     resolver::update(ctx)?;
            //     kaspad::update(ctx)?;
            //     Ok(false)
            // }
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
