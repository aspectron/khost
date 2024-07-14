use crate::imports::*;
use actions::*;

#[derive(Describe, Eq, PartialEq, Debug, Clone, Copy)]
#[caption = "Main menu"]
pub enum Main {
    /// Configure services
    #[describe("Manage")]
    Manage,
    /// Software updates
    #[describe("Updates")]
    Update,
    /// Uninstall services, delete data, etc.
    #[describe("Advanced")]
    Advanced,
    /// Exit the program
    Exit,
}

impl Action for Main {
    fn main(&self, ctx: &mut Context) -> Result<bool> {
        match self {
            Main::Manage => {
                Manage::select(ctx)?;
                Ok(true)
            }
            Main::Update => {
                Update::select(ctx)?;
                Ok(true)
            }
            Main::Advanced => {
                Advanced::select(ctx)?;
                Ok(true)
            }
            Main::Exit => {
                outro("bye!")?;
                println!();
                std::process::exit(0);
            }
        }
    }
}
