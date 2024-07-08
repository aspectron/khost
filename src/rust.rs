use crate::imports::*;

pub fn update() -> Result<()> {
    static RUSTUP_UPDATED: AtomicBool = AtomicBool::new(false);
    if !RUSTUP_UPDATED.load(Ordering::Relaxed) {
        cmd!("rustup", "update", "stable").run()?;
    }
    Ok(())
}
