use crate::imports::*;

#[derive(Describe, Eq, PartialEq, Debug, Clone, Copy)]
#[caption = "Select services to update"]
pub enum Update {
    /// Go back to the previous menu
    #[describe("Back")]
    Back,
    #[describe("Update all services")]
    All,
}

impl Action<Context> for Update {
    type Error = Error;
    fn run(&self, ctx: &mut Context) -> Result<()> {
        match self {
            Update::All => {
                if confirm("This will install the Kaspa software and configure services. Continue?")
                    .interact()?
                {
                    bootstrap::run(ctx)?;

                    nginx::install(ctx)?;
                    resolver::install(ctx)?;
                    kaspad::install(ctx)?;
                }

                Ok(())
            }
            Update::Back => Ok(()),
        }
    }
}
