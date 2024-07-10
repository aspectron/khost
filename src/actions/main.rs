use crate::imports::*;
use actions::*;

#[derive(Describe, Eq, PartialEq, Debug, Clone, Copy)]
#[caption = "Main menu"]
pub enum Main {
    #[describe("Configure services")]
    Configure,
    /// Display service status
    #[describe("Status and logs")]
    Status,
    /// Enable or disable services
    #[describe("Service control")]
    Manage,
    /// Update services
    #[describe("Software updates")]
    Update,
    /// Uninstall services
    #[describe("Advanced")]
    Advanced,
    /// Exit the program
    Exit,
}

impl Action<Context> for Main {
    type Error = Error;
    fn run(&self, ctx: &mut Context) -> Result<()> {
        match self {
            Main::Configure => {
                Configure::select()?.run(ctx)?;
            }

            Main::Status => {
                Status::select()?.run(ctx)?;
            }

            Main::Manage => {
                // TODO - multi-select service to enable/disable
            }
            Main::Update => {
                Update::select()?.run(ctx)?;
            }
            Main::Advanced => {
                Advanced::select()?.run(ctx)?;
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
