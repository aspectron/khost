use crate::imports::*;

#[derive(Describe, Eq, PartialEq, Debug, Clone, Copy)]
#[caption = "First time install"]
pub enum Bootstrap {
    /// Perform default public node installation
    #[describe("Default install")]
    Default,
    /// Continue to main menu
    #[describe("Skip")]
    Skip,
}

impl Action for Bootstrap {
    fn main(&self, ctx: &mut Context) -> Result<bool> {
        match self {
            Bootstrap::Default => {
                if confirm("This will install the Kaspa software and configure services. Continue?")
                    .initial_value(true)
                    .interact()?
                {
                    if confirm("Would you like to install Kaspa resolver?").interact()? {
                        ctx.config.resolver.enabled = true;
                        resolver::init_resolver_config(ctx).ok();
                    }

                    kaspad::select_networks(ctx)?;

                    bootstrap::run(ctx)?;
                    ctx.config.bootstrap = true;
                    ctx.config.save()?;

                    nginx::install(ctx)?;
                    resolver::install(ctx)?;
                    kaspad::install(ctx)?;
                }

                Ok(false)
            }
            Bootstrap::Skip => {
                ctx.config.bootstrap = true;
                ctx.config.save()?;

                log::info("You can perform a full install later from the Advanced menu.")?;

                Ok(false)
            }
        }
    }
}
