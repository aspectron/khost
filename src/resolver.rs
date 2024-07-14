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
        ServiceDetail::new(
            "Kaspa RPC resolver",
            SERVICE_NAME,
            ServiceKind::Resolver,
            self.enabled,
            true,
        )
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

    pub fn enabled(&self) -> bool {
        self.enabled
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

pub fn is_installed(ctx: &Context) -> bool {
    let origin = &ctx.config.resolver.origin;
    folder(origin).exists() && binary(&ctx.config.resolver.origin).exists()
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
    systemd::enable(service_name(config))?;
    systemd::start(service_name(config))?;

    nginx::create(nginx_config(ctx))?;
    nginx::reload()?;

    Ok(())
}

pub fn nginx_config(ctx: &Context) -> NginxConfig {
    let config = &ctx.config.resolver;
    let fqdns = fqdn::get(false);
    let server_kind = ServerKind::http().with_fqdn(fqdns);
    let proxy_kind = ProxyKind::http(8989);
    let proxy_config = ProxyConfig::new("/", proxy_kind);
    NginxConfig::new(service_name(config), server_kind, vec![proxy_config])
}

pub fn update(ctx: &Context) -> Result<()> {
    let config = &ctx.config.resolver;

    if !config.enabled {
        return Ok(());
    }

    fetch(&config.origin)?;
    build(&config.origin)?;
    restart(config)?;
    Ok(())
}

pub fn uninstall(ctx: &mut Context) -> Result<()> {
    let config = &mut ctx.config.resolver;
    config.enabled = false;

    log::remark("Uninstalling resolver...")?;

    let service_name = service_name(config);

    if nginx::exists(&service_name) {
        nginx::remove(&service_name)?;
        nginx::reload()?;
    } else {
        log::error(format!("Nginx config file '{service_name}' not found"))?;
    }

    if systemd::exists(&service_name) {
        systemd::stop(&service_name)?;
        systemd::disable(&service_name)?;
        systemd::remove(&service_name)?;
    } else {
        log::error(format!("Systemd unit file '{service_name}' not found"))?;
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
        log::success(format!("Build successful for version {version}"))?;
        Ok(())
    } else {
        log::error("Build error: unable to determine resolver version")?;
        Err(Error::custom("Failed to execute resolver"))
    }
}

pub fn version(origin: &Origin) -> Option<String> {
    duct::cmd!(binary(origin), "--version")
        .stderr_to_stdout()
        .unchecked()
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

    let unit_config = systemd::Config::new(
        service_name(config),
        "Kaspa Resolver",
        &ctx.username,
        exec_start,
        5,
    );

    systemd::create(unit_config)?;

    Ok(())
}

pub fn restart(config: &Config) -> Result<()> {
    step("Restarting resolver...", || {
        systemd::restart(service_name(config))
    })
}

pub fn status(config: &Config) -> Result<String> {
    systemd::status(service_name(config))
}
