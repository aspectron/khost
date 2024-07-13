use std::sync::{Arc, Mutex};

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
                let args: std::vec::Vec<OsString> = std::vec!["-kS".into(),"-p".into(),"".into(),$program.into(),$( Into::<OsString>::into($arg) ),*];

                let password = $crate::sudo::password().expect("missing user password") + "\n";
                $crate::cmd::cmd("sudo", args).stdin_bytes(password.as_bytes())
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
        sudo!("mv", "-f", temp, path.as_ref()).run()?;
        Ok(())
    }

    pub fn remove_file<P: AsRef<Path>>(path: P) -> Result<()> {
        sudo!("rm", "-f", path.as_ref()).run()?;
        Ok(())
    }
}
