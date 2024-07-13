use crate::imports::*;

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "interface", content = "port")]
#[serde(rename_all = "lowercase")]
pub enum Interface {
    Public(u16),
    Local(u16),
}

impl Display for Interface {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            Interface::Public(port) => write!(f, "0.0.0.0:{}", port),
            Interface::Local(port) => write!(f, "127.0.0.1:{}", port),
        }
    }
}

impl Interface {
    pub fn port(&self) -> u16 {
        match self {
            Interface::Public(port) => *port,
            Interface::Local(port) => *port,
        }
    }
}

#[derive(Default, Describe, Clone, Copy, Debug, Serialize, Deserialize, Eq, PartialEq, Hash)]
#[serde(rename_all = "lowercase")]
pub enum Network {
    #[default]
    Mainnet,
    Testnet10,
    Testnet11,
}

impl Display for Network {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            Network::Mainnet => write!(f, "mainnet"),
            Network::Testnet10 => write!(f, "testnet-10"),
            Network::Testnet11 => write!(f, "testnet-11"),
        }
    }
}
