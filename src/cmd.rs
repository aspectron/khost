// use duct::*;
use crate::result::Result;
use std::ffi::OsString;
use std::path::PathBuf;
use std::sync::OnceLock;

static VERBOSE: OnceLock<bool> = OnceLock::new();

#[inline]
pub fn init_verbose_mode(verbose: bool) {
    VERBOSE.set(verbose).unwrap();
}

#[inline]
pub fn verbose() -> bool {
    *VERBOSE.get().expect("verbose mode is not initialized")
}

#[inline]
pub fn not_verbose() -> bool {
    !*VERBOSE.get().expect("verbose mode is not initialized")
}

pub mod macros {

    #[macro_export]
    macro_rules! cmd {
        ( $program:expr $(, $arg:expr )* $(,)? ) => {
            {
                use std::ffi::OsString;
                let args: std::vec::Vec<OsString> = std::vec![$( Into::<OsString>::into($arg) ),*];
                $crate::cmd::cmd($program, args)
            }
        };
    }

    // #[macro_export]
    // macro_rules! sudo {
    //     ( $program:expr $(, $arg:expr )* $(,)? ) => {
    //         {
    //             // use duct::Expression;
    //             use std::ffi::OsString;
    //             let args: std::vec::Vec<OsString> = std::vec!["-kS".into(),$program.into(),$( Into::<OsString>::into($arg) ),*];

    //             let password = $crate::cmd::password().expect("missing user password") + "\n";
    //             $crate::cmd::cmd("sudo", args).stdin_bytes(password.as_bytes())
    //         }
    //     };
    // }

    pub use cmd;
    // pub use sudo;
}

pub struct Expression(duct::Expression);

impl Expression {
    pub fn run(&self) -> Result<()> {
        use std::io::Read;

        if !verbose() {
            let mut reader = self.0.reader()?;

            let mut output = String::new();
            match reader.read_to_string(&mut output) {
                Ok(_) => (),
                Err(e) => {
                    while output.ends_with('\n') || output.ends_with('\r') {
                        output.truncate(output.len() - 1);
                    }
                    println!();
                    println!();
                    println!("{}", output);
                    println!();
                    return Err(e.into());
                }
            }
        } else {
            self.0.run()?;
        }

        Ok(())
    }

    pub fn read(&self) -> Result<String> {
        Ok(self.0.read()?)
    }

    pub fn dir(&mut self, dir: impl Into<PathBuf>) -> &mut Self {
        self.0 = self.0.dir(dir);
        self
    }

    pub fn unchecked(&self) -> Self {
        Self(self.0.unchecked())
    }

    pub fn stdin_bytes<T: Into<Vec<u8>>>(&self, bytes: T) -> Self {
        Self(self.0.stdin_bytes(bytes))
    }

    pub fn inner(self) -> duct::Expression {
        self.0
    }
}

pub fn cmd<T, U>(program: T, args: U) -> Expression
where
    T: duct::IntoExecutablePath,
    U: IntoIterator,
    U::Item: Into<OsString>,
{
    let expr = duct::cmd(program, args).stderr_to_stdout();
    Expression(expr)
}
