use crate::imports::*;
use nginx::prelude::*;

pub const SERVICE_NAME: &str = "kaspa-resolver";

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Config {
    pub enabled: bool,
    pub origin: Origin,
    pub sync: bool,
    pub stats: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub http: Option<Interface>,
}

impl Service for Config {
    fn service_detail(&self) -> ServiceDetail {
        ServiceDetail::new("Kaspa RPC resolver", SERVICE_NAME)
    }
}

impl Config {
    pub fn new(origin: Origin) -> Self {
        Self {
            enabled: false,
            origin,
            sync: false,
            stats: true,
            http: None,
        }
    }

    pub fn with_stats(self) -> Self {
        Self {
            stats: true,
            ..self
        }
    }

    pub fn with_public_interface(self, port: u16) -> Self {
        Self {
            http: Some(Interface::Public(port)),
            ..self
        }
    }

    pub fn with_local_interface(self, port: u16) -> Self {
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

pub fn fetch(origin: &Origin) -> Result<()> {
    let path = folder(origin);

    track("Synchronizing resolver sources...", |step| {
        if path.exists() {
            git::restore(&path, origin)?;
            git::pull(&path, origin)?;
        } else {
            git::clone(&path, origin)?;
        }

        step.done("Resolver sources synchronized...")
    })?;

    Ok(())
}

pub fn binary(origin: &Origin) -> PathBuf {
    folder(origin).join("target/release/kaspa-resolver")
}

pub fn folder(origin: &Origin) -> PathBuf {
    base_folder().join(origin.folder())
}

pub fn base_folder() -> PathBuf {
    root_folder().join("kaspa-resolver")
}

pub fn install(ctx: &mut Context) -> Result<()> {
    if !ctx.config.resolver.enabled {
        return Ok(());
    }

    log::remark("Installing Kaspa wPRC resolver...")?;

    let config = &ctx.config.resolver;

    fetch(&config.origin)?;
    build(&config.origin)?;

    create_systemd_unit(ctx, config)?;
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

pub fn update(ctx: &Context) -> Result<()> {
    let config = &ctx.config.resolver;

    if !config.enabled {
        return Ok(());
    }

    fetch(&config.origin)?;
    build(&config.origin)?;
    restart()?;
    Ok(())
}

pub fn uninstall() -> Result<()> {
    log::remark("Uninstalling resolver...")?;

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

    let path = base_folder();
    if path.exists() {
        log::info("Removing resolver...")?;
        fs::remove_dir_all(&path)?;
        log::success("Resolver removed")?;
    } else {
        log::error("Resolver folder not found")?;
    }

    Ok(())
}

pub fn build(origin: &Origin) -> Result<()> {
    rust::update()?;

    step("Building resolver...", || {
        cmd!("cargo", "build", "--release")
            .dir(folder(origin))
            .run()
    })?;

    if let Some(version) = version(origin) {
        log::success("Build successful")?;
        log::info(format!("Resolver version: {}", version))?;
        Ok(())
    } else {
        log::error("Unable to determine resolver version")?;
        Err(Error::custom("Failed to execute resolver"))
    }
}

pub fn version(origin: &Origin) -> Option<String> {
    cmd!(binary(origin), "--version")
        .read()
        .ok()
        .and_then(|s| s.trim().split(' ').last().map(String::from))
}

pub fn create_systemd_unit(ctx: &Context, config: &Config) -> Result<()> {
    let args = Vec::<String>::from(config);
    let exec_start = [binary(&config.origin).display().to_string()]
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

pub fn status() -> Result<String> {
    systemd::status(SERVICE_NAME)
}

pub fn is_active() -> Result<bool> {
    systemd::is_active(SERVICE_NAME)
}

pub fn is_enabled() -> Result<bool> {
    systemd::is_enabled(SERVICE_NAME)
}
