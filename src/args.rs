pub use clap::Parser;

#[derive(Default, Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Args {
    /// Verbose mode
    #[arg(short, long, default_value = "false")]
    pub verbose: bool,
}

pub fn parse() -> Args {
    let args = Args::parse();

    crate::cmd::init_verbose_mode(args.verbose);

    args
}
