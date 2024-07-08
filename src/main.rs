pub mod actions;
pub mod base;
pub mod bootstrap;
pub mod config;
pub mod content;
pub mod context;
pub mod error;
pub mod folders;
pub mod fqdn;
pub mod git;
pub mod imports;
pub mod kaspad;
pub mod khost;
pub mod network;
pub mod nginx;
pub mod resolver;
pub mod result;
pub mod rust;
pub mod service;
pub mod status;
pub mod system;
pub mod systemd;

use crate::imports::*;

// #[tokio::main]
fn main() {
    println!();

    if runtime::is_windows() {
        let _ = log::error("kHOST supports Linux OS only");
        let _ = cliclack::outro("Exiting...");
        println!();
        std::process::exit(1);
    }

    // Check for updates
    khost::update().ok();

    ctrlc::set_handler(move || {}).expect("setting Ctrl-C handler");

    let mut ctx = Context::try_new().unwrap();

    let first_run = !ctx.config.bootstrap;

    // bootstrap::check(&mut ctx);

    let status = status::detect(&ctx);
    let _ = cliclack::note(format!("kHOST v{}", khost::VERSION), &status);

    if let Some(conflicts) = status::conflicts(&ctx, &status) {
        conflicts.iter().for_each(|c| {
            let _ = log::error(c);
        });
    }

    if first_run {
        if let Err(err) = Bootstrap::select("First time install")
            .map_err(Into::into)
            .and_then(|choice| choice.run(&mut ctx))
        {
            log::error(err).ok();
            log::info("You can attempt another full install from 'Advanced' menu").ok();
        }
    }

    loop {
        if let Err(e) = run(&mut ctx) {
            match e {
                Error::Sudo => {
                    log::error(
                        "This command requires root privileges\nPlease exit and run `sudo khost`",
                    )
                    .ok();
                }
                Error::Io(e) if e.kind() == std::io::ErrorKind::Interrupted => {
                    println!();
                    std::process::exit(0);
                }
                e => {
                    // println!("Error: {:?}", e);
                    log::error(&e).ok();
                }
            }
        }
    }
}

fn run(ctx: &mut Context) -> Result<()> {
    Main::select("Select an action")?.run(ctx)?;
    Ok(())
}
