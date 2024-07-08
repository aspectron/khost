use crate::imports::*;

#[derive(Describe, Eq, PartialEq, Debug, Clone, Copy)]
pub enum Bootstrap {
    /// Perform default public node installation
    #[describe("Default install")]
    Default,
    /// Continue to main menu
    #[describe("Skip")]
    Skip,
}

impl Action<Context> for Bootstrap {
    type Error = Error;
    fn run(&self, ctx: &mut Context) -> Result<()> {
        match self {
            Bootstrap::Default => {
                if cliclack::confirm(
                    "This will install the Kaspa software and configure services. Continue?",
                )
                .interact()?
                {
                    bootstrap::run(ctx)?;

                    nginx::install(ctx)?;
                    resolver::install(ctx)?;
                    kaspad::install(ctx)?;
                }

                Ok(())
            }
            Bootstrap::Skip => Ok(()),
        }
    }
}
