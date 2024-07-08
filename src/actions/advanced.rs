use crate::imports::*;

#[derive(Describe, Eq, PartialEq, Debug, Clone, Copy)]
pub enum Advanced {
    #[describe("Go Back")]
    Back,
    #[describe("Uninstall Kaspa software")]
    Uninstall,
}
