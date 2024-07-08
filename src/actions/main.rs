use crate::imports::*;

#[derive(Describe, Eq, PartialEq, Debug, Clone, Copy)]
pub enum Main {
    #[describe("Configure services")]
    Configure,
    #[describe("Display service status")]
    Status,
    #[describe("Display service logs")]
    Logs,
    #[describe("Enable or disable services")]
    Enable,
    // / Update
    #[describe("Updates")]
    Upgrade,
    #[describe("Advanced")]
    Advanced,
    // /// Rebuilds with latest sources
    // #[describe("Update services")]
    // Remove,
    /// Exit the program
    Exit,
}

impl Action<Context> for Main {
    type Error = Error;
    fn run(&self, _ctx: &mut Context) -> Result<()> {
        match self {
            Main::Configure => {}

            Main::Status => {}

            Main::Logs => {}

            Main::Enable => {
                // ctx.root()?;
                // status(&context)
                cliclack::note("Status", "not implemented\nhello world")?;
            }
            Main::Upgrade => {
                // ctx.root()?;
                // ls(&context)
                // cliclack::note("Ls", "not implemented\nhello world")?;
                log::step("ls -la")?;
                // cmd!("ls","-la").full_env(std::env::vars()).run()?;
                cmd!("ls", "-la").run()?;
                // println!("│");
                // log::step("")?;

                // log::step("bash ls -la")?;
                // cmd!("bash ls -la").full_env(std::env::vars()).run().ok();
                // log::step("cargo --version")?;
                // cmd!("cargo --version").full_env(std::env::vars()).run().ok();
                // log::step("bash cargo --version")?;
                // cmd!("bash cargo --version").full_env(std::env::vars()).run().ok();
            }
            Main::Advanced => {
                // ctx.root()?;
                // advanced(&context)
                // cliclack::note("Advanced", "not implemented\nhello world")?;
            }
            // Main::Install => {
            //     // ctx.root()?;

            //     kaspad::fetch()?;
            //     kaspad::build()?;
            //     // install(&context)
            // }
            // Main::Upgrade => {
            //     ctx.root()?;
            //     // upgrade(&context)
            // }
            // Main::Domain => {
            //     ctx.root()?;
            //     // domain(&context)
            // }
            Main::Exit => {
                // context.exit()?;
                cliclack::outro("bye!")?;
                println!();
                std::process::exit(0);
            }
        }

        Ok(())
    }
}
