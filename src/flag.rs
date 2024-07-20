use crate::imports::*;

pub fn exists(flag: &str) -> bool {
    data_folder().join(flag).exists()
}

pub fn create(flag: &str) -> Result<()> {
    fs::write(data_folder().join(flag), "")?;
    Ok(())
}

pub fn remove(flag: &str) -> Result<()> {
    fs::remove_file(data_folder().join(flag))?;
    Ok(())
}
