use crate::imports::*;

#[derive(Describe, Eq, PartialEq, Debug, Clone, Copy)]
#[caption = "Advanced menu"]
pub enum Advanced {
    /// Go back to the previous menu
    #[describe("Back")]
    Back,
    /// Perform full installation
    #[describe("Full installation")]
    Full,
    #[describe("Uninstall Kaspa software")]
    Uninstall,
}

impl Action<Context> for Advanced {
    type Error = Error;
    fn run(&self, ctx: &mut Context) -> Result<()> {
        match self {
            Advanced::Back => {}
            Advanced::Full => {
                actions::Bootstrap::select()?.run(ctx)?;
            }
            Advanced::Uninstall => {
                if confirm("Are you sure you want to uninstall Kaspa software?").interact()? {
                    log::step("Uninstalling Kaspa software")?;
                    resolver::uninstall()?;
                    kaspad::uninstall(ctx)?;
                    log::success("Kaspa software uninstalled successfully")?;
                }
            }
        }

        Ok(())
    }
}
