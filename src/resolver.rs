use crate::imports::*;
use nginx::prelude::*;

const SERVICE_NAME: &str = "kaspa-resolver";

#[derive(Default, Clone, Debug, Serialize, Deserialize)]
pub struct Config {
    stats: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    http: Option<Interface>,
}

impl Config {
    fn with_stats(self) -> Self {
        Self {
            stats: true,
            ..self
        }
    }

    // fn with_public_interface(self, port: u16) -> Self {
    //     Self { http: Some(Interface::Public(port)), ..self }
    // }

    fn with_local_interface(self, port: u16) -> Self {
        Self {
            http: Some(Interface::Local(port)),
            ..self
        }
    }
}

impl From<&Config> for Vec<String> {
    fn from(config: &Config) -> Self {
        let mut args = Arglist::default();

        if config.stats {
            args.push("--stats");
        }

        if let Some(interface) = config.http.as_ref() {
            args.push(format!("--listen={interface}"));
        }

        args.into()
    }
}

pub fn fetch() -> Result<()> {
    let path = folder();

    log::info("Synchronizing resolver sources...")?;
    if path.exists() {
        git::restore(&path)?;
        git::pull(&path)?;
    } else {
        git::clone("https://github.com/aspectron/kaspa-resolver", &path, None)?;
    }
    log::success("Resolver sources synchronized...")?;

    Ok(())
}

pub fn binary() -> PathBuf {
    folder().join("target/release/kaspa-resolver")
}

pub fn folder() -> PathBuf {
    root_folder().join("kaspa-resolver")
}

pub fn install(ctx: &mut Context) -> Result<()> {
    fetch()?;
    build()?;

    let config = Config::default().with_stats().with_local_interface(8989);

    create_systemd_unit(ctx, &config)?;
    systemd::daemon_reload()?;
    systemd::enable(SERVICE_NAME)?;
    systemd::start(SERVICE_NAME)?;

    nginx::create(nginx_config(ctx))?;
    nginx::reload()?;

    Ok(())
}

pub fn nginx_config(_ctx: &Context) -> NginxConfig {
    let fqdns = fqdn::get(false);
    let server_kind = ServerKind::http().with_fqdn(fqdns);
    let proxy_kind = ProxyKind::http(8989);
    let proxy_config = ProxyConfig::new("/", proxy_kind);
    NginxConfig::new(SERVICE_NAME, server_kind, vec![proxy_config])
}

pub fn update() -> Result<()> {
    fetch()?;
    build()?;
    restart()?;
    Ok(())
}

pub fn uninstall() -> Result<()> {
    if nginx::exists(SERVICE_NAME) {
        nginx::remove(SERVICE_NAME)?;
        nginx::reload()?;
    } else {
        log::error(format!("Nginx config file '{SERVICE_NAME}' not found"))?;
    }

    if systemd::exists(SERVICE_NAME) {
        systemd::stop(SERVICE_NAME)?;
        systemd::disable(SERVICE_NAME)?;
        systemd::remove(SERVICE_NAME)?;
    } else {
        log::error(format!("Systemd unit file '{SERVICE_NAME}' not found"))?;
    }

    let path = folder();
    if path.exists() {
        log::info("Removing resolver...")?;
        fs::remove_dir_all(&path)?;
        log::success("Resolver removed")?;
    } else {
        log::error("Resolver folder not found")?;
    }

    Ok(())
}

pub fn build() -> Result<()> {
    rust::update()?;

    log::info("Building resolver...")?;

    cmd("cargo", &["build", "--release"]).dir(folder()).run()?;

    if let Some(version) = version() {
        log::success("Build successful")?;
        log::info(format!("Resolver version: {}", version))?;
        Ok(())
    } else {
        log::error("Unable to determine resolver version")?;
        Err(Error::custom("Failed to execute resolver"))
    }
}

pub fn version() -> Option<String> {
    cmd!(binary(), "--version")
        .read()
        .ok()
        .and_then(|s| s.trim().split(' ').last().map(String::from))
}

pub fn create_systemd_unit(ctx: &Context, config: &Config) -> Result<()> {
    let args = Vec::<String>::from(config);
    let exec_start = [binary().display().to_string()]
        .into_iter()
        .chain(args)
        .collect::<Vec<_>>();

    let unit_config =
        systemd::Config::new(SERVICE_NAME, "Kaspa Resolver", &ctx.username, exec_start, 5);

    systemd::create(unit_config)?;

    Ok(())
}

pub fn start() -> Result<()> {
    systemd::start(SERVICE_NAME)?;
    Ok(())
}

pub fn stop() -> Result<()> {
    systemd::stop(SERVICE_NAME)?;
    Ok(())
}

pub fn restart() -> Result<()> {
    log::info("Restarting resolver...")?;
    systemd::restart(SERVICE_NAME)
}

// pub fn status() -> Result<String> {
//     systemd::status("kaspa-resolver")
// }

pub fn logs() -> Result<()> {
    systemd::logs(SERVICE_NAME)
}

pub fn is_active() -> Result<bool> {
    systemd::is_active(SERVICE_NAME)
}

// pub fn enable() -> Result<()> {
//     systemd::enable("kaspa-resolver")
// }

pub fn is_enabled() -> Result<bool> {
    systemd::is_enabled(SERVICE_NAME)
}

// pub fn exists() -> bool {
//     systemd::exists("kaspa-resolver")
// }

pub fn update_config() -> Result<()> {
    let key = load_key()?;
    let data = reqwest::blocking::get(
        "https://raw.githubusercontent.com/aspectron/kaspa-resolver/master/resolver.bin",
    )?
    .bytes()?;
    // let data = http::get_bytes(
    //     "https://raw.githubusercontent.com/aspectron/kaspa-resolver/master/resolver.bin",
    // )
    // .await?;
    let _ = chacha20poly1305::decrypt_slice(&data, &key.into())?;
    fs::write(config_folder().join("resolver.bin"), data)?;
    Ok(())
}

pub fn config_folder() -> PathBuf {
    home_folder().join(".kaspa-resolver")
}

pub fn create_key(passphrase: Secret) -> Result<()> {
    let config_folder = config_folder();
    fs::create_dir_all(&config_folder)?;
    let hash = argon2_sha256(passphrase.as_bytes(), 32)?;
    fs::write(config_folder.join("key"), hash)?;
    Ok(())
}

fn load_key() -> Result<Vec<u8>> {
    let config_folder = config_folder();
    Ok(fs::read(config_folder.join("key"))?)
}

// pub fn create_nginx_config(ctx: &Context) -> Result<()> {
//     let tpl = ctx.tpl.local([
//         ("ROOT", folder(ctx).display().to_string().as_str()),
//         ("PORT", "8080"),
//     ]);

//     let config = tpl.render("nginx/kaspa-resolver.conf")?;
//     let path = PathBuf::from("/etc/nginx/sites-available/kaspa-resolver.conf");

//     fs::write(&path, config)?;
//     Ok(())
// }
