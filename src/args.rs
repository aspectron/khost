use crate::imports::*;
pub use clap::Parser;

#[derive(Default, Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Args {
    /// Verbose mode
    #[arg(short, long, default_value = "false")]
    pub verbose: bool,
    /// Reset configuration
    #[arg(short, long, default_value = "false")]
    pub reset: bool,
}

pub fn parse() -> Args {
    let args = Args::parse();

    crate::cmd::init_verbose_mode(args.verbose);

    if args.reset {
        Config::reset();
        log::info("Configuration reset").ok();
        outro("have a great day!").ok();
        std::process::exit(0);
    }

    args
}
