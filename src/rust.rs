use crate::imports::*;

pub fn update() -> Result<()> {
    static RUSTUP_UPDATED: AtomicBool = AtomicBool::new(false);
    if !RUSTUP_UPDATED.load(Ordering::Relaxed) {
        step("Updating Rust Compiler (stable)...", || {
            cmd!("rustup", "update", "stable").run()?;
            RUSTUP_UPDATED.store(true, Ordering::Relaxed);
            Ok(())
        })?;
    }

    Ok(())
}

pub fn check() -> Result<()> {
    if PathBuf::from("/usr/bin/rustc").exists() {
        log::error("System-wide installation of Rust compiler is detected.\nThis is not correct and can interfere with software updates.")?;
        if confirm("Uninstall system-wide rust compiler?")
            .initial_value(true)
            .interact()?
        {
            step("Uninstalling Rust compiler...", || {
                sudo!("apt", "remove", "rustc", "-y").run()?;
                Ok(())
            })?;
            step("Setting up user-local Rust compiler...", || {
                let rustup = reqwest::blocking::get("https://sh.rustup.rs")?.bytes()?;
                fs::write("/tmp/rustup.sh", rustup)?;
                cmd!(
                    "sh",
                    "/tmp/rustup.sh",
                    "-q",
                    "-y",
                    "--default-toolchain",
                    "stable"
                )
                .run()?;
                Ok(())
            })?;
        }
    }
    Ok(())
}
