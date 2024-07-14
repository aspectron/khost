use crate::imports::*;

#[derive(Describe, Eq, PartialEq, Debug, Clone, Copy)]
#[caption = "Configure"]
pub enum Configure {
    /// Go back to the previous menu
    #[describe("Back")]
    Back,
    // #[describe("Fail")]
    // Fail,
    // #[describe("Verbose mode")]
    // Verbose,
    /// Enable or disable services
    #[describe("Enable services")]
    Enable,
}

impl Action for Configure {
    fn main(&self, ctx: &mut Context) -> Result<bool> {
        match self {
            // Configure::Fail => {
            //     cmd!("bash", "fail").run()?;
            // }
            Configure::Back => Ok(false),
            // Configure::Verbose => {
            //     if confirm("Enable verbose mode?")? {
            //         ctx.config.verbose = true;
            //         ctx.config.save()?;
            //     }
            // }
            Configure::Enable => {
                let services = ctx.managed_services();
                let active = ctx.managed_active_services();
                let mut selector =
                    cliclack::multiselect("Select services to enable (ESC to cancel)")
                        .initial_values(active);
                for service in services {
                    selector = selector.item(service.clone(), service, "");
                }
                match selector.interact() {
                    Ok(services) => {
                        enable_services(ctx, services)?;
                    }
                    Err(_) => {
                        println!();
                    }
                }
                Ok(true)
            }
        }
    }
}
