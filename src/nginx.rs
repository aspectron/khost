// use std::io::Read;

use crate::imports::*;

pub mod prelude {
    pub use super::{Certs, Config as NginxConfig, ProxyConfig, ProxyKind, ServerKind};
}

const NGINX_CONFIG_PATH: &str = "/etc/nginx/";

pub struct Certs {
    pub key: String,
    pub crt: String,
}

pub enum ServerKind {
    Http {
        port: Option<u16>,
        fqdns: Vec<String>,
    },
    Ssl {
        port: Option<u16>,
        certs: Certs,
        fqdns: Vec<String>,
    },
}

impl ServerKind {
    pub fn http() -> Self {
        Self::Http {
            port: Default::default(),
            fqdns: vec![],
        }
    }

    pub fn with_port(mut self, port: u16) -> Self {
        match &mut self {
            Self::Http { port: p, .. } => *p = Some(port),
            Self::Ssl { port: p, .. } => *p = Some(port),
        }

        self
    }

    pub fn with_fqdn<S: Display>(mut self, fqdn: Vec<S>) -> Self {
        let fqdns_ = fqdn
            .iter()
            .map(|fqdn| fqdn.to_string())
            .collect::<Vec<String>>();
        match &mut self {
            Self::Http { fqdns, .. } => fqdns.extend(fqdns_),
            Self::Ssl { fqdns, .. } => fqdns.extend(fqdns_),
        }

        self
    }

    pub fn with_certs<S: Display>(mut self, key: S, crt: S) -> Self {
        match &mut self {
            Self::Ssl { certs, .. } => {
                certs.key = key.to_string();
                certs.crt = crt.to_string();
            }
            _ => {
                panic!("Cannot set certificates for non-SSL server");
            }
        }

        self
    }
}

pub enum ProxyKind {
    Http { port: u16 },
    Wrpc { port: u16 },
}

impl ProxyKind {
    pub fn http(port: u16) -> Self {
        Self::Http { port }
    }

    pub fn wrpc(port: u16) -> Self {
        Self::Wrpc { port }
    }
}

pub struct ProxyConfig {
    pub path: String,
    pub proxy_kind: ProxyKind,
}

impl ProxyConfig {
    pub fn new<S: Display>(path: S, proxy_kind: ProxyKind) -> Self {
        Self {
            path: path.to_string(),
            proxy_kind,
        }
    }
}

pub struct Config {
    pub service_name: String,
    pub server_kind: ServerKind,
    pub proxy_config: Vec<ProxyConfig>,
}

impl Config {
    pub fn new<S: Display>(
        service_name: S,
        server_kind: ServerKind,
        proxy_config: Vec<ProxyConfig>,
    ) -> Self {
        Self {
            service_name: service_name.to_string(),
            server_kind,
            proxy_config,
        }
    }
}

impl Display for Config {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        writeln!(f, "# {}", self.service_name)?;
        writeln!(f, "server {{")?;

        match &self.server_kind {
            ServerKind::Http { port, fqdns } => {
                let port = port.unwrap_or(80);
                writeln!(f, "\tlisten {port};")?;
                writeln!(f, "\tlisten [::]:{port};")?;
                writeln!(f, "\tserver_name {};", fqdn::flatten(fqdns))?;
            }
            ServerKind::Ssl { port, certs, fqdns } => {
                let port = port.unwrap_or(443);
                writeln!(f, "\tlisten {port} ssl;")?;
                writeln!(f, "\tlisten [::]:{port} ssl;")?;
                writeln!(f, "\tssl_certificate {}", certs.crt)?;
                writeln!(f, "\tssl_certificate_key {}", certs.key)?;
                writeln!(f, "\tserver_name {};", fqdn::flatten(fqdns))?;
            }
        }

        writeln!(f, "\tclient_max_body_size 1m;")?;

        for proxy in self.proxy_config.iter() {
            let ProxyConfig {
                path,
                proxy_kind: proxy,
            } = proxy;
            writeln!(f, "\tlocation {path} {{")?;
            writeln!(f, "\t\tproxy_http_version 1.1;")?;
            writeln!(f, "\t\tproxy_set_header Host $host;")?;
            writeln!(f, "\t\tproxy_set_header X-Real-IP $remote_addr;")?;

            match proxy {
                ProxyKind::Http { port } => {
                    writeln!(f, "\t\tproxy_pass http://127.0.0.1:{port}/;")?;
                }
                ProxyKind::Wrpc { port } => {
                    writeln!(f, "\t\tproxy_set_header Upgrade $http_upgrade;")?;
                    writeln!(f, "\t\tproxy_set_header Connection \"Upgrade\";")?;
                    writeln!(f, "\t\tproxy_pass http://127.0.0.1:{port}/;")?;
                }
            }

            writeln!(f, "\t}}")?;
            writeln!(f)?;
        }

        writeln!(f, "}}")?;
        writeln!(f)?;

        Ok(())
    }
}

pub fn version() -> Option<String> {
    cmd!("nginx", "-v")
        .read()
        .ok()
        .map(|s| s.trim().to_string())
}

pub fn install(_ctx: &Context) -> Result<()> {
    step("Installing Nginx...", || {
        sudo!("apt", "install", "-y", "nginx").run()
    })?;

    Ok(())
}

pub fn reload() -> Result<()> {
    step("Reloading Nginx...", || {
        sudo!("nginx", "-s", "reload").run()
    })
}

pub fn reconfigure() -> Result<()> {
    // TODO
    // let nginx_config = fs::read_to_string(PathBuf::from(NGINX_CONFIG_PATH).join("nginx.conf"))?;
    Ok(())
}

pub fn config_filename<S>(service_name: S) -> PathBuf
where
    S: Display,
{
    PathBuf::from(NGINX_CONFIG_PATH).join(format!("sites-enabled/{service_name}.conf"))
}

pub fn create(config: Config) -> Result<()> {
    let config_filename = config_filename(&config.service_name);
    log::step(format!(
        "Creating Nginx config file: '{}'",
        config_filename.display()
    ))?;
    sudo::fs::write(config_filename, config.to_string())?;
    Ok(())
}

pub fn remove<S: Display>(service_name: S) -> Result<()> {
    let config_filename = config_filename(service_name);
    log::step(format!(
        "Removing Nginx config file: '{}'",
        config_filename.display()
    ))?;
    sudo::fs::remove_file(config_filename)?;
    Ok(())
}

pub fn exists(service_name: &str) -> bool {
    config_filename(service_name).exists()
}
