use crate::imports::*;

pub fn clone<U: Display, P: AsRef<Path>>(url: U, path: P, branch: Option<&str>) -> Result<()> {
    let url = url.to_string();
    let path = path.as_ref().display().to_string();

    if let Some(branch) = branch {
        cmd("git", &["clone", "-b", branch, &url, &path]).run()?;
    } else {
        cmd("git", &["clone", &url, &path]).run()?;
    }

    Ok(())
}

pub fn pull<P: AsRef<Path>>(path: P) -> Result<()> {
    let path = path.as_ref().display().to_string();

    cmd("git", &["pull", &path]).dir(path).run()?;

    Ok(())
}

pub fn restore<P: AsRef<Path>>(path: P) -> Result<()> {
    let path = path.as_ref().display().to_string();

    cmd("git", &["restore", &path]).dir(path).run()?;

    Ok(())
}

pub fn version() -> Option<String> {
    cmd!("git", "--version")
        .read()
        .ok()
        .map(|s| s.trim().to_string())
}
