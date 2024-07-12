use crate::imports::*;
use nginx::prelude::*;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Config {
    enabled: bool,
    network: Network,
    data_folder: Option<PathBuf>,
    enable_upnp: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    grpc: Option<Interface>,
    #[serde(skip_serializing_if = "Option::is_none")]
    wrpc: Option<Interface>,
}

impl Service for Config {
    fn service_name(&self) -> String {
        format!("kaspa-{}", self.network)
    }
}

impl Config {
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    pub fn data_folder(&self) -> PathBuf {
        self.data_folder.clone().unwrap_or_else(|| {
            home_folder()
                .join(".rusty-kaspa")
                .join(service_name(&self.network))
        })
    }

    pub fn network(&self) -> Network {
        self.network
    }
}

impl From<Network> for Config {
    fn from(network: Network) -> Self {
        let (grpc, wrpc) = match network {
            Network::Mainnet => (16110, 17110),
            Network::Testnet10 => (16210, 17210),
            Network::Testnet11 => (16310, 17310),
        };

        Config {
            enabled: false,
            network,
            data_folder: None,
            enable_upnp: true,
            grpc: Some(Interface::Local(grpc)),
            wrpc: Some(Interface::Public(wrpc)),
        }
    }
}

impl From<&Config> for Vec<String> {
    fn from(config: &Config) -> Self {
        let mut args = Arglist::default();

        match config.network {
            Network::Mainnet => {}
            Network::Testnet10 => {
                args.push("--testnet");
                args.push("--netsuffix=10");
            }
            Network::Testnet11 => {
                args.push("--testnet");
                args.push("--netsuffix=11");
            }
        }

        args.push("--perf-metrics");
        args.push("--perf-metrics-interval-sec=1");
        args.push("--yes");
        args.push("--utxoindex");

        if !config.enable_upnp {
            args.push("--disable-upnp");
        }

        match config.grpc {
            Some(Interface::Public(port)) => {
                args.push(format!("--rpclisten=0.0.0.0:{port}"));
            }
            Some(Interface::Local(port)) => {
                args.push(format!("--rpclisten=127.0.0.1:{port}"));
            }
            None => {
                args.push("--nogrpc");
            }
        }

        match config.wrpc {
            Some(Interface::Public(port)) => {
                args.push(format!("--rpclisten-borsh=0.0.0.0:{port}"));
            }
            Some(Interface::Local(port)) => {
                args.push(format!("--rpclisten-borsh=127.0.0.1:{port}"));
            }
            None => {}
        }

        if let Some(data_folder) = &config.data_folder {
            args.push(format!("--appdir={}", data_folder.display()));
        }

        args.into()
    }
}

pub fn active_configs(ctx: &Context) -> impl Iterator<Item = &Config> {
    ctx.config
        .kaspad
        .iter()
        .filter(|config| config.is_enabled())
}

pub fn inactive_configs(ctx: &Context) -> impl Iterator<Item = &Config> {
    ctx.config
        .kaspad
        .iter()
        .filter(|config| !config.is_enabled())
}

pub fn fetch() -> Result<()> {
    let path = folder();

    if path.exists() {
        git::restore(&path)?;
        git::pull(&path)?;
    } else {
        git::clone(
            "https://github.com/aspectron/rusty-kaspa",
            &path,
            Some("omega"),
        )?;
    }

    Ok(())
}

pub fn install(ctx: &mut Context) -> Result<()> {
    fetch()?;
    build()?;

    reconfigure(ctx, true)?;

    Ok(())
}

pub fn nginx_config(_ctx: &Context, config: &Config) -> NginxConfig {
    let fqdns = fqdn::get(false);
    let server_kind = ServerKind::http().with_fqdn(fqdns);
    let port = config
        .wrpc
        .as_ref()
        .expect("expecting Kaspad wRPC interface")
        .port();
    let proxy_kind = ProxyKind::wrpc(port);
    let proxy_config = ProxyConfig::new(format!("/{}", config.network), proxy_kind);
    NginxConfig::new(service_name(config), server_kind, vec![proxy_config])
}

pub fn update(ctx: &Context) -> Result<()> {
    fetch()?;
    build()?;
    for config in active_configs(ctx) {
        restart(config)?;
    }
    Ok(())
}

pub fn uninstall(ctx: &Context) -> Result<()> {
    for config in active_configs(ctx) {
        let service_name = service_name(config);
        log::remark(format!("Uninstalling Kaspad p2p node '{service_name}'..."))?;

        if systemd::exists(&service_name) {
            systemd::stop(&service_name)?;
            systemd::disable(&service_name)?;
            systemd::remove(&service_name)?;
        } else {
            log::error(format!("Systemd unit file '{service_name}' not found"))?;
        }

        if nginx::exists(&service_name) {
            nginx::remove(&service_name)?;
            nginx::reload()?;
        } else {
            log::error(format!("Nginx config file '{service_name}' not found"))?;
        }
    }

    let path = folder();
    if path.exists() {
        step("Removing Rusty Kaspa p2p node...", || {
            fs::remove_dir_all(&path)?;
            Ok(())
        })?;
    } else {
        log::error("Rusty Kaspa folder not found")?;
    }

    if confirm("Do you want to remove Kaspa p2p node data folder?").interact()? {
        for config in ctx.config.kaspad.iter() {
            let data_folder = if let Some(data_folder) = &config.data_folder {
                data_folder.clone()
            } else {
                home_folder().join(".kaspad")
            };

            let network_folder = data_folder.join(config.network.to_string());
            if network_folder.exists() {
                step(
                    format!(
                        "Removing Kaspa p2p node data folder: '{}'",
                        network_folder.display()
                    ),
                    || {
                        fs::remove_dir_all(&network_folder)?;
                        Ok(())
                    },
                )?;
            } else {
                log::error(format!(
                    "Kaspa p2p node data folder not found: '{}'",
                    network_folder.display()
                ))?;
            }
        }
    }

    Ok(())
}

pub fn build() -> Result<()> {
    rust::update()?;
    
    step("Building Kaspad p2p node...", || {
        cmd!("cargo", "build", "--release", "--bin", "kaspad")
            .dir(folder())
            .run()
    })?;

    if let Some(version) = version() {
        log::success("Build successful")?;
        log::info(format!("Kaspad version: {}", version))?;
        Ok(())
    } else {
        log::error("Unable to determine kaspad version")?;
        Err(Error::custom("Failed to execute kaspad"))
    }
}

pub fn binary() -> PathBuf {
    folder().join("target/release/kaspad")
}

pub fn folder() -> PathBuf {
    root_folder().join("rusty-kaspa")
}

pub fn version() -> Option<String> {
    duct::cmd!(binary(), "--version")
        .read()
        .ok()
        .and_then(|s| s.trim().split(' ').last().map(String::from))
}

pub fn create_systemd_unit(ctx: &Context, config: &Config) -> Result<()> {
    let service_name = service_name(config);
    let description = format!("Kaspad p2p Node ({})", config.network);

    let args = Vec::<String>::from(config);
    let exec_start = [binary().display().to_string()]
        .into_iter()
        .chain(args)
        .collect::<Vec<_>>();

    let unit_config = systemd::Config::new(service_name, description, &ctx.username, exec_start, 5);

    systemd::create(unit_config)?;
    Ok(())
}

pub fn start(config: &Config) -> Result<()> {
    systemd::start(service_name(config))
}

pub fn stop(config: &Config) -> Result<()> {
    systemd::stop(service_name(config))
}

pub fn restart(config: &Config) -> Result<()> {
    systemd::restart(service_name(config))
}

pub fn status(config: &Config) -> Result<String> {
    systemd::status(service_name(config))
}

pub fn logs(config: &Config) -> Result<()> {
    systemd::logs(service_name(config))
}

pub fn is_active(config: &Config) -> Result<bool> {
    systemd::is_active(service_name(config))
}

pub fn is_enabled(config: &Config) -> Result<bool> {
    systemd::is_enabled(service_name(config))
}

pub fn configure_networks(ctx: &mut Context, networks: Vec<Network>) -> Result<()> {
    let networks = networks.into_iter().collect::<HashSet<_>>();
    if networks.len() > 1 && ctx.system.ram_as_gb() < 17 {
        log::warning(format!(
            "Detected RAM is {}, minimum required for multiple networks is 32 Gb.",
            as_gb(ctx.system.total_memory as f64, false, false)
        ))?;
        if !confirm("Continue with multiple network setup?").interact()? {
            log::warning("Aborting...")?;
            return Ok(());
        }
    }

    for config in ctx.config.kaspad.iter_mut() {
        config.enabled = networks.contains(&config.network);
    }
    ctx.config.save()?;

    reconfigure(ctx, false)?;

    Ok(())
}

pub fn reconfigure(ctx: &Context, force: bool) -> Result<()> {
    let mut reconfigure_systemd = false;
    let mut reconfigure_nginx = false;

    log::remark("Updating Kaspa p2p node configuration...")?;

    for config in inactive_configs(ctx) {
        let service_name = service_name(config);
        if systemd::exists(&service_name) {
            if systemd::is_active(&service_name)? {
                systemd::stop(&service_name)?;
            }
            systemd::disable(&service_name)?;
            systemd::remove(&service_name)?;
            reconfigure_systemd = true;
        }

        if nginx::exists(&service_name) {
            nginx::remove(&service_name)?;
            reconfigure_nginx = true;
        }
    }

    for config in active_configs(ctx) {
        let service_name = service_name(config);

        if force || !systemd::exists(&service_name) {
            create_systemd_unit(ctx, config)?;
            reconfigure_systemd = true;
        }

        if force || !nginx::exists(&service_name) {
            nginx::create(nginx_config(ctx, config))?;
            reconfigure_nginx = true;
        }
    }

    if reconfigure_systemd {
        systemd::daemon_reload()?;

        for config in active_configs(ctx) {
            let service_name = service_name(config);
            systemd::enable(&service_name)?;
            systemd::start(&service_name)?;
        }
    }

    if reconfigure_nginx {
        nginx::reload()?;
    }

    Ok(())
}

pub fn stop_all(ctx: &Context) -> Result<()> {
    for config in active_configs(ctx) {
        stop(config)?;
    }
    Ok(())
}

pub fn start_all(ctx: &Context) -> Result<()> {
    for config in active_configs(ctx) {
        start(config)?;
    }
    Ok(())
}

pub fn purge_data_folder_all(ctx: &Context) -> Result<()> {
    for config in active_configs(ctx) {
        purge_data_folder(config)?;
    }
    Ok(())
}

pub fn purge_data_folder(config: &Config) -> Result<()> {
    let data_folder = config.data_folder();

    step(
        format!(
            "Removing Kaspa p2p node data folder: '{}'",
            data_folder.display()
        ),
        || {
            fs::remove_dir_all(&data_folder)?;
            Ok(())
        },
    )
}
