use crate::imports::*;
use actions::*;

#[derive(Describe, Eq, PartialEq, Debug, Clone, Copy)]
#[caption = "Main menu"]
pub enum Main {
    /// Configure services
    #[describe("Configure")]
    Configure,
    /// Display service status
    #[describe("Status")]
    Status,
    /// Enable or disable services
    #[describe("Control")]
    Control,
    /// Update services to the latest version
    #[describe("Updates")]
    Update,
    /// Uninstall services, delete data, etc.
    #[describe("Advanced")]
    Advanced,
    /// Exit the program
    Exit,
}

impl Action for Main {
    fn main(&self, ctx: &mut Context) -> Result<()> {
        match self {
            Main::Configure => {
                Configure::select(ctx)?;
            }

            Main::Status => {
                Status::select(ctx)?;
            }

            Main::Control => {
                // TODO - multi-select service to enable/disable
            }
            Main::Update => {
                Update::select(ctx)?;
            }
            Main::Advanced => {
                Advanced::select(ctx)?;
            }
            Main::Exit => {
                cliclack::outro("bye!")?;
                println!();
                std::process::exit(0);
            }
        }

        Ok(())
    }
}
