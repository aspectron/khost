use crate::imports::*;
use nginx::prelude::*;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Config {
    enabled: bool,
    origin: Origin,
    network: Network,
    data_folder: Option<PathBuf>,
    enable_upnp: bool,
    outgoing_peers: Option<u16>,
    max_incoming_peers: Option<u16>,
    #[serde(skip_serializing_if = "Option::is_none")]
    grpc: Option<Interface>,
    #[serde(skip_serializing_if = "Option::is_none")]
    wrpc_borsh: Option<Interface>,
    #[serde(skip_serializing_if = "Option::is_none")]
    wrpc_json: Option<Interface>,
}

impl Service for Config {
    fn service_detail(&self) -> ServiceDetail {
        ServiceDetail::new(
            "Kaspa p2p node",
            format!("kaspa-{}", self.network),
            ServiceKind::Kaspad(self.network),
            self.enabled,
            true,
        )
    }
}

impl Config {
    pub fn new(origin: Origin, network: Network) -> Self {
        let (grpc, wrpc_borsh, wrpc_json) = match network {
            Network::Mainnet => (16110, 17110, 18110),
            Network::Testnet10 => (16210, 17210, 18210),
            Network::Testnet11 => (16310, 17310, 18310),
        };

        Self {
            enabled: false,
            origin,
            network,
            data_folder: None,
            enable_upnp: false,
            outgoing_peers: Some(32),
            max_incoming_peers: Some(1024),
            grpc: Some(Interface::Local(grpc)),
            wrpc_borsh: Some(Interface::Local(wrpc_borsh)),
            wrpc_json: Some(Interface::Local(wrpc_json)),
        }
    }

    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    pub fn data_folder(&self) -> PathBuf {
        self.data_folder
            .clone()
            .unwrap_or_else(|| home_folder().join(".rusty-kaspa").join(service_name(self)))
    }

    pub fn network(&self) -> Network {
        self.network
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

        args.push("--yes");
        args.push("--perf-metrics");
        args.push("--perf-metrics-interval-sec=1");
        args.push("--utxoindex");
        args.push("--loglevel=info,kaspad_lib::daemon=trace ");

        if !config.enable_upnp {
            args.push("--disable-upnp");
        }

        if let Some(outgoing_peers) = config.outgoing_peers {
            args.push(format!("--outpeers={outgoing_peers}"));
        }

        if let Some(max_incoming_peers) = config.max_incoming_peers {
            args.push(format!("--maxinpeers={max_incoming_peers}"));
        }

        if let Some(interface) = &config.grpc {
            args.push(format!("--rpclisten={interface}"));
        }

        if let Some(interface) = &config.wrpc_borsh {
            args.push(format!("--rpclisten-borsh={interface}"));
        }

        if let Some(interface) = &config.wrpc_json {
            args.push(format!("--rpclisten-json={interface}"));
        }

        if let Some(data_folder) = &config.data_folder {
            args.push(format!("--appdir={}", data_folder.display()));
        }

        args.into()
    }
}

pub fn unique_origins(ctx: &Context) -> HashSet<Origin> {
    ctx.config
        .kaspad
        .iter()
        .map(|config| config.origin.clone())
        .collect()
}

// pub fn active_unique_origins(ctx: &Context) -> HashSet<Origin> {
//     ctx.config
//         .kaspad
//         .iter()
//         .filter_map(|config| config.enabled.then_some(config.origin.clone()))
//         .collect()
// }

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

pub fn fetch(ctx: &Context) -> Result<()> {
    for origin in unique_origins(ctx) {
        let path = folder(&origin);
        if path.exists() {
            git::reset(&path)?;
            git::pull(&path)?;
        } else {
            git::clone(&path, &origin)?;
        }
    }

    Ok(())
}

pub fn install(ctx: &mut Context) -> Result<()> {
    fetch(ctx)?;
    build(ctx)?;

    reconfigure(ctx, true)?;

    Ok(())
}

pub fn is_installed(ctx: &Context) -> bool {
    unique_origins(ctx)
        .iter()
        .all(|origin| binary(origin).exists())
}

pub fn nginx_config(_ctx: &Context, config: &Config) -> NginxConfig {
    let fqdns = fqdn::get(false);
    let server_kind = ServerKind::http().with_fqdn(fqdns);

    let mut proxy_configs = Vec::new();

    if let Some(iface) = config.wrpc_borsh.as_ref() {
        let port = iface.port();
        let proxy_kind = ProxyKind::wrpc(port);
        let proxy_config =
            ProxyConfig::new(format!("/kaspa/{}/wrpc/borsh", config.network), proxy_kind);
        proxy_configs.push(proxy_config);
    }

    if let Some(iface) = config.wrpc_json.as_ref() {
        let port = iface.port();
        let proxy_kind = ProxyKind::wrpc(port);
        let proxy_config =
            ProxyConfig::new(format!("/kaspa/{}/wrpc/json", config.network), proxy_kind);
        proxy_configs.push(proxy_config);
    }

    NginxConfig::new(service_name(config), server_kind, proxy_configs)
}

pub fn update(ctx: &Context) -> Result<()> {
    fetch(ctx)?;
    build(ctx)?;
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

    let path = base_folder();
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

pub fn build(ctx: &Context) -> Result<()> {
    rust::update()?;

    for origin in unique_origins(ctx) {
        let folder = folder(&origin);

        step(format!("Building Kaspad p2p node ({})", origin), || {
            cmd!("cargo", "build", "--release", "--bin", "kaspad")
                .dir(&folder)
                .run()
        })?;

        if let Some(version) = version(&origin) {
            log::success(format!("Build successful for version {version}"))?;
        } else {
            log::error("Build error: unable to determine kaspad version")?;
        }
    }

    Ok(())
}

pub fn binary(origin: &Origin) -> PathBuf {
    folder(origin).join("target/release/kaspad")
}

pub fn folder(origin: &Origin) -> PathBuf {
    base_folder().join(origin.folder())
}

pub fn base_folder() -> PathBuf {
    root_folder().join("rusty-kaspa")
}

pub fn version(origin: &Origin) -> Option<String> {
    let hash = git::hash(folder(origin), true).unwrap_or("unknown".to_string());

    duct::cmd!(binary(origin), "--version")
        .stderr_to_stdout()
        .unchecked()
        .read()
        .ok()
        .and_then(|s| {
            s.trim()
                .split(' ')
                .last()
                .map(|version| format!("{version}-{hash}"))
        })
}

pub fn create_systemd_unit(ctx: &Context, config: &Config) -> Result<()> {
    let service_name = service_name(config);
    let description = format!("Kaspad p2p Node ({})", config.network);

    let args = Vec::<String>::from(config);
    let exec_start = [binary(&config.origin).display().to_string()]
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
                step(format!("Bringing down '{}'", service_name), || {
                    systemd::stop(&service_name)
                })?;
            }
            step(format!("Removing service '{}'", service_name), || {
                systemd::disable(&service_name)?;
                systemd::remove(&service_name)?;
                reconfigure_systemd = true;
                Ok(())
            })?;
        }

        if nginx::exists(&service_name) {
            nginx::remove(&service_name)?;
            reconfigure_nginx = true;
        }
    }

    for config in active_configs(ctx) {
        let service_name = service_name(config);
        step(format!("Configuring '{}'", service_name), || {
            if force || !systemd::exists(&service_name) {
                create_systemd_unit(ctx, config)?;
                reconfigure_systemd = true;
            }

            if force || !nginx::exists(&service_name) {
                nginx::create(nginx_config(ctx, config))?;
                reconfigure_nginx = true;
            }

            Ok(())
        })?;
    }

    if reconfigure_systemd {
        step("Reloading systemd daemon...", systemd::daemon_reload)?;

        for config in active_configs(ctx) {
            let service_name = service_name(config);
            step(format!("Brining up '{}'", service_name), || {
                systemd::enable(&service_name)?;
                systemd::start(&service_name)
            })?;
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

pub fn check_for_updates(ctx: &Context) -> Result<()> {
    let mut updates = Vec::new();
    for origin in unique_origins(ctx) {
        let path = folder(&origin);

        let latest = git::latest_commit_hash(&origin, true)?;
        let current = git::hash(path, true)?;
        if latest != current {
            log::info(format!(
                "Kaspad p2p node update available ({origin}): {current} -> {latest}"
            ))?;
            updates.push((origin, current, latest));
        }
    }

    if !updates.is_empty()
        && confirm("Update Kaspad p2p node?")
            .initial_value(true)
            .interact()?
    {
        update(ctx)?;
    }

    Ok(())
}
