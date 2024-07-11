pub mod actions;
pub mod args;
pub mod base;
pub mod bootstrap;
pub mod config;
pub mod console;
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
#[macro_use]
pub mod cmd;
pub use cmd::*;

use crate::imports::*;

// #[tokio::main]
fn main() {
    println!();

    if !is_root() {
        cliclack::note(
            format!("kHOST v{}", khost::VERSION),
            "kHOST requires root privileges\nPlease run `sudo khost`",
        )
        .ok();
        std::process::exit(1);
    }

    let args = args::parse();

    if runtime::is_windows() {
        let _ = log::error("kHOST supports Linux OS only");
        let _ = cliclack::outro("Exiting...");
        println!();
        std::process::exit(1);
    }

    // Check for updates
    khost::update().ok();

    let mut ctx = Context::try_new(args).unwrap();

    let first_run = !ctx.config.bootstrap;
    // let first_run = true;

    // bootstrap::check(&mut ctx);

    let status = status::detect(&ctx);
    let _ = cliclack::note(format!("kHOST v{}", khost::VERSION), &status);

    status::conflicts(&ctx, &status);
    // if let Some(conflicts) = status::conflicts(&ctx, &status) {
    //     conflicts.iter().for_each(|c| {
    //         let _ = log::error(c);
    //     });
    // }

    init_user_interaction();

    if first_run {
        if let Err(err) = actions::Bootstrap::select(&mut ctx) {
            log::error(err).ok();
            log::info("You can attempt another full install from 'Advanced' menu").ok();
        }
    }

    actions::Main::run(&mut ctx).ok();

    println!();
}
