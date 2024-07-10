use crate::imports::*;

pub const VERSION: &str = env!("CARGO_PKG_VERSION");

pub fn binary() -> Result<PathBuf> {
    Ok(std::env::current_exe()?)
}

pub fn version() -> version::Version {
    VERSION.parse().unwrap()
}

pub fn is_cargo_install() -> bool {
    binary().unwrap().display().to_string().contains(".cargo")
}

pub fn update() -> Result<()> {
    if is_cargo_install() {
        if let Ok(latest_version) = version::blocking::latest_crate_version("khost") {
            if latest_version.is_greater_than(version()) {
                log::warning(format!("New version of khost@{latest_version} detected"))?;
                if confirm("Would you like to update?").interact()? {
                    log::info(format!("Updating khost to {latest_version}"))?;
                    cmd!("cargo", "install", format!("khost@{latest_version}")).run()?;
                    log::success(format!("khost updated to {latest_version}"))?;
                    log::info("Starting new version...")?;
                    println!();
                    surrender();
                }
            }
        }
    }

    Ok(())
}

pub fn surrender() {
    let _ = cmd!("khost").run();
    std::process::exit(0);
}
