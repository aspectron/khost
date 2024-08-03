pub mod actions;
pub mod args;
pub mod base;
pub mod config;
pub mod console;
pub mod content;
pub mod context;
pub mod error;
pub mod flag;
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
pub mod tls;
pub mod utils;
#[macro_use]
pub mod cmd;
pub use cmd::*;
#[macro_use]
pub mod sudo;

use crate::imports::*;

// #[tokio::main]
fn main() {
    println!();

    init_theme();

    let args = args::parse();

    if runtime::is_windows() {
        let _ = log::error("kHOST supports Linux OS only");
        let _ = outro("Exiting...");
        println!();
        std::process::exit(1);
    }

    if is_root() {
        let _ = log::error("kHOST should not be run as root");
        let _ = outro("Exiting...");
        println!();
        std::process::exit(2);
    }

    // Check for updates
    khost::update().ok();

    // init context & load khost config
    let mut ctx = Context::try_new(args).unwrap();

    sudo::init(&mut ctx);

    let first_run = !ctx.config.bootstrap;

    let status = status::detect(&ctx);
    let _ = cliclack::note(format!("kHOST v{}", khost::VERSION), &status);

    status::conflicts(&ctx, &status);

    init_user_interaction();

    let services_updated = if first_run {
        if let Err(err) = actions::Bootstrap::select(&mut ctx) {
            log::error(err).ok();
            log::info("You can attempt another full install from 'Advanced' menu").ok();
        }
        false
    } else {
        let kaspad_update = kaspad::check_for_updates(&ctx).unwrap_or_default();
        let resolver_update = resolver::check_for_updates(&ctx).unwrap_or_default();
        kaspad_update || resolver_update
    };

    if let Err(err) = khost::reconfigure_if_needed(&mut ctx, services_updated) {
        log::error(err).ok();
    }

    actions::Main::run(&mut ctx).ok();
    // if let Err(err) = actions::Main::run(&mut ctx) {
    //     outro(style(err.to_string()).red().bright()).ok();
    // }

    println!();
}
