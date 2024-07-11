use crate::error::Error;
pub type Result<T> = std::result::Result<T, Error>;

pub trait Capture<T> {
    fn capture(self) -> Result<T>;
}

impl Capture<()> for Result<()> {
    fn capture(self) -> Result<()> {
        match self {
            Err(Error::Io(e)) if e.kind() == std::io::ErrorKind::Interrupted => Ok(()),
            Err(e) => Err(e),
            Ok(r) => Ok(r),
        }
    }
}
