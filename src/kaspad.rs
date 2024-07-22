use crate::imports::*;
use nginx::prelude::*;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Config {
    enabled: bool,
    #[serde(default)]
    certs: Option<Certs>,
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
    fn service_title(&self) -> String {
        "Kaspa p2p node".to_string()
    }

    fn service_name(&self) -> String {
        format!("kaspa-{}", self.network)
    }

    fn kind(&self) -> ServiceKind {
        ServiceKind::Kaspad(self.network)
    }

    fn origin(&self) -> Option<Origin> {
        Some(self.origin.clone())
    }

    fn enabled(&self) -> bool {
        self.enabled
    }

    fn managed(&self) -> bool {
        true
    }

    fn proxy_config(&self, _ctx: &Context) -> Option<Vec<ProxyConfig>> {
        let mut proxy_configs = Vec::new();

        if let Some(iface) = self.wrpc_borsh.as_ref() {
            let port = iface.port();
            let proxy_kind = ProxyKind::wrpc(port);
            let proxy_config = ProxyConfig::new(
                format!("{} ({})", self.service_title(), self.service_name()),
                format!("/kaspa/{}/wrpc/borsh", self.network),
                proxy_kind,
            );
            proxy_configs.push(proxy_config);
        }

        if let Some(iface) = self.wrpc_json.as_ref() {
            let port = iface.port();
            let proxy_kind = ProxyKind::wrpc(port);
            let proxy_config = ProxyConfig::new(
                format!("{} ({})", self.service_title(), self.service_name()),
                format!("/kaspa/{}/wrpc/json", self.network),
                proxy_kind,
            );
            proxy_configs.push(proxy_config);
        }

        Some(proxy_configs)
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
            certs: None,
            origin,
            network,
            data_folder: None,
            enable_upnp: false,
            outgoing_peers: Some(32),
            max_incoming_peers: Some(256),
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
            .unwrap_or_else(|| home_folder().join(".rusty-kaspa").join(self.service_name()))
    }

    pub fn network(&self) -> Network {
        self.network
    }

    pub fn enable(&mut self) {
        self.enabled = true;
    }

    pub fn disable(&mut self) {
        self.enabled = false;
    }

    pub fn set_origin(&mut self, origin: Origin) {
        self.origin = origin;
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

pub fn update(ctx: &Context) -> Result<()> {
    fetch(ctx)?;
    build(ctx)?;
    step("Restarting Kaspa p2p nodes...", || {
        for config in active_configs(ctx) {
            systemd::restart(config)?;
        }
        Ok(())
    })?;
    log::success("Update successful")?;
    Ok(())
}

pub fn uninstall(ctx: &Context) -> Result<()> {
    for config in active_configs(ctx) {
        let service_name = config.service_name();
        log::remark(format!("Uninstalling Kaspad p2p node '{service_name}'..."))?;

        if systemd::exists(config) {
            systemd::stop(config)?;
            systemd::disable(config)?;
            systemd::remove(config)?;
        } else {
            log::error(format!("Systemd unit file '{service_name}' not found"))?;
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
    let description = format!("Kaspad p2p Node ({})", config.network);

    let args = Vec::<String>::from(config);
    let exec_start = [binary(&config.origin).display().to_string()]
        .into_iter()
        .chain(args)
        .collect::<Vec<_>>();

    let unit_config = systemd::Config::new(config, description, &ctx.username, exec_start, 5);

    systemd::create(unit_config)?;
    Ok(())
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

    log::remark("Updating Kaspa p2p node configuration...")?;

    for config in inactive_configs(ctx) {
        let service_name = config.service_name();
        if systemd::exists(config) {
            if systemd::is_active(&config.service_name())? {
                step(format!("Bringing down '{}'", service_name), || {
                    systemd::stop(config)
                })?;
            }
            step(format!("Removing service '{}'", service_name), || {
                systemd::disable(config)?;
                systemd::remove(config)?;
                reconfigure_systemd = true;
                Ok(())
            })?;
        }
    }

    for config in active_configs(ctx) {
        let service_name = config.service_name();
        step(format!("Configuring '{}'", service_name), || {
            if force || !systemd::exists(config) {
                create_systemd_unit(ctx, config)?;
                reconfigure_systemd = true;
            }
            Ok(())
        })?;
    }

    if reconfigure_systemd {
        step("Reloading systemd daemon...", systemd::daemon_reload)?;

        for config in active_configs(ctx) {
            let service_name = config.service_name();
            step(format!("Brining up '{}'", service_name), || {
                systemd::enable(config)?;
                systemd::start(config)
            })?;
        }
    }

    log::success("Kaspa p2p node configuration updated")?;

    Ok(())
}

pub fn stop_all(ctx: &Context) -> Result<()> {
    for config in active_configs(ctx) {
        systemd::stop(config)?;
    }
    Ok(())
}

pub fn start_all(ctx: &Context) -> Result<()> {
    for config in active_configs(ctx) {
        systemd::start(config)?;
    }
    Ok(())
}

pub fn restart_all(ctx: &Context) -> Result<()> {
    for config in active_configs(ctx) {
        step(
            format!(
                "Restarting {} ({})",
                config.service_title(),
                config.service_name()
            ),
            || {
                systemd::restart(config)?;
                Ok(())
            },
        )?;
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

pub fn find_config_by_network<'a>(
    ctx: &'a mut Context,
    network: &Network,
) -> Option<&'a mut Config> {
    ctx.config
        .kaspad
        .iter_mut()
        .find(|config| config.network == *network)
}

pub fn find_config_by_service_detail<'a>(
    ctx: &'a mut Context,
    detail: &ServiceDetail,
) -> Option<&'a mut Config> {
    ctx.config
        .kaspad
        .iter_mut()
        .find(|config| config.service_name() == detail.name)
}

pub fn select_networks(ctx: &mut Context) -> Result<()> {
    if ctx.system.total_memory < 15 * 1024 * 1024 * 1024 {
        log::warning(format!(
            "Detected RAM is {}, minimum required for multiple networks is 32 Gb.",
            as_gb(ctx.system.total_memory as f64, false, false)
        ))?;

        let mut selector = cliclack::select("Select Kaspa p2p node network to enable");
        let details = ctx
            .config
            .kaspad
            .iter()
            .map(Service::service_detail)
            .collect::<Vec<_>>();
        let selected = active_configs(ctx).next().map(Service::service_detail);
        if let Some(selected) = selected {
            selector = selector.initial_value(selected);
        }
        for detail in details {
            selector = selector.item(detail.clone(), detail, "");
        }
        let selected = selector.interact()?;
        ctx.config.kaspad.iter_mut().for_each(Config::disable);
        find_config_by_service_detail(ctx, &selected)
            .unwrap()
            .enable();
    } else {
        let details = ctx
            .config
            .kaspad
            .iter()
            .map(Service::service_detail)
            .collect::<Vec<_>>();
        let enabled = details
            .iter()
            .filter(|config| config.enabled)
            .cloned()
            .collect::<Vec<_>>();
        let mut selector = cliclack::multiselect("Select Kaspa p2p node networks to enable");
        if !enabled.is_empty() {
            selector = selector.initial_values(enabled);
        }
        for detail in details {
            selector = selector.item(detail.clone(), detail, "");
        }
        let selected = selector.interact()?;
        ctx.config.kaspad.iter_mut().for_each(Config::disable);
        for detail in selected {
            find_config_by_service_detail(ctx, &detail)
                .unwrap()
                .enable();
        }
    }
    Ok(())
}
