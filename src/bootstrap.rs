use crate::imports::*;

// #[derive(Default)]
// pub struct State {
//     pub base: bool,

// }

pub fn check(ctx: &mut Context) {
    if !ctx.config.bootstrap {
        if let Err(err) = run(ctx) {
            println!();
            let _ = log::error(err);
            println!();
            std::process::exit(1);
        }
    }
}

pub fn run(ctx: &mut Context) -> Result<()> {
    if runtime::is_linux() {
        base::install(ctx)?;
    }

    Ok(())
}
