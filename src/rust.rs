use crate::imports::*;

pub fn update() -> Result<()> {
    static RUSTUP_UPDATED: AtomicBool = AtomicBool::new(false);
    if !RUSTUP_UPDATED.load(Ordering::Relaxed) {
        step("Building Kaspad p2p node...", || {
            cmd!("rustup", "update", "stable").run()?;
            RUSTUP_UPDATED.store(true, Ordering::Relaxed);
            Ok(())
        })?;
    }
    Ok(())
}
