use crate::imports::*;
use std::sync::{Arc, Mutex};
// use duct::*;

lazy_static::lazy_static! {
    pub static ref PASSWORD: Arc<Mutex<Option<String>>> = Arc::new(Mutex::new(None));
}

pub fn init_password(password: String) {
    *PASSWORD.lock().unwrap() = Some(password);
}

pub fn password() -> Option<String> {
    PASSWORD.lock().unwrap().clone()
}

pub mod macros {

    #[macro_export]
    macro_rules! sudo {
        ( $program:expr $(, $arg:expr )* $(,)? ) => {
            {
                use std::ffi::OsString;

                if let Some(password) = $crate::sudo::password() {
                    let password = password + "\n";
                    let args: std::vec::Vec<OsString> = std::vec!["-kS".into(),"-p".into(),"".into(),$program.into(),$( Into::<OsString>::into($arg) ),*];
                    $crate::cmd::cmd("sudo", args).stdin_bytes(password.as_bytes())
                } else {
                    let args: std::vec::Vec<OsString> = std::vec![$program.into(),$( Into::<OsString>::into($arg) ),*];
                    $crate::cmd::cmd("sudo", args)
                }
            }
        };
    }

    pub use sudo;
}

pub mod fs {
    use crate::imports::*;

    pub fn write<P, C>(path: P, content: C) -> Result<()>
    where
        P: AsRef<Path>,
        C: AsRef<[u8]>,
    {
        let temp = temp_folder().join("temp.txt");
        fs::write(&temp, content)?;
        sudo!("chown", "root:root", &temp).run()?;
        sudo!("chmod", "644", &temp).run()?;
        sudo!("mv", "-f", temp, path.as_ref()).run()?;
        Ok(())
    }

    pub fn remove_file<P: AsRef<Path>>(path: P) -> Result<()> {
        sudo!("rm", "-f", path.as_ref()).run()?;
        Ok(())
    }
}

fn is_sudo_with_password() -> bool {
    duct::cmd!("sudo", "-n", "true")
        .stderr_to_stdout()
        .read()
        .is_err()
}

#[inline]
fn sudoers_entry_path() -> PathBuf {
    Path::new("/etc/sudoers.d").join("khost")
}

pub fn create_sudoers_entry(ctx: &Context) -> Result<()> {
    // let sudoers = Path::new("/etc/sudoers");
    let sudoers_entry_path = sudoers_entry_path();
    if sudoers_entry_path.exists() {
        log::error(format!(
            "Sudoers configuration already exists at `{}`",
            sudoers_entry_path.display()
        ))?;
    } else {
        let config = format!("{} ALL=ALL NOPASSWD: ALL\n", ctx.username);
        fs::write(&sudoers_entry_path, config)?;
    }
    Ok(())
}

pub fn remove_sudoers_entry() -> Result<()> {
    let sudoers_entry_path = sudoers_entry_path();
    if sudoers_entry_path.exists() {
        fs::remove_file(sudoers_entry_path)?;
    }
    Ok(())
}

pub fn toggle_sudoers_entry(ctx: &mut Context) -> Result<()> {
    if sudoers_entry_path().exists() {
        remove_sudoers_entry()?;
        query_sudo_password();
    } else {
        query_sudo_password();
        create_sudoers_entry(ctx)?;
    }
    Ok(())
}

pub fn disable_sudo_password(ctx: &Context) -> Result<()> {
    if confirm(format!(
        "Would you like to disable sudo password prompt for user '{}'?\n\n\
Disabling sudo password prompt allows khost (and sudo)\n\
to stop asking for password. This is a security risk.\n\
Proceed with caution and make sure your password is strong.\n
",
        ctx.username
    ))
    .interact()?
    {
        create_sudoers_entry(ctx)?;
    }
    Ok(())
}

fn query_sudo_password() {
    if password().is_some() {
        return;
    }

    loop {
        match cliclack::password("Enter user password:").interact() {
            Ok(password) => {
                if duct::cmd!("sudo", "-kS", "-p", "", "echo", "khost")
                    .stdin_bytes(password.as_bytes())
                    .stderr_to_stdout()
                    .read()
                    .is_ok()
                {
                    sudo::init_password(password);
                    break;
                } else {
                    log::error("Invalid password").ok();
                }
            }
            Err(e) => {
                log::error(e.to_string()).ok();
                outro("Exiting...").ok();
                std::process::exit(1);
            }
        }
    }
}

pub fn init(ctx: &mut Context) {
    if !is_sudo_with_password() {
        return;
    }

    query_sudo_password();

    if !ctx.config.disable_sudo_prompt {
        log::warning("Root access (sudo) requires user password prompt").ok();
        disable_sudo_password(ctx).ok();
        ctx.config.disable_sudo_prompt = true;
        ctx.config.save().ok();
    }
}
