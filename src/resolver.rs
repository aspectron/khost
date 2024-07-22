use crate::imports::*;
use nginx::prelude::*;

pub const SERVICE_NAME: &str = "kaspa-resolver";

#[derive(Describe, Clone, Copy, Debug, Serialize, Deserialize, Eq, PartialEq)]
pub enum ResolverKind {
    /// contributor-maintained public network
    #[describe("Public Kaspa node network")]
    Public,
    /// dedicated high-availability cluster
    #[describe("Private Kaspa node cluster")]
    Private,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Config {
    pub enabled: bool,
    pub kind: Option<ResolverKind>,
    #[serde(default)]
    pub certs: Option<Certs>,
    pub origin: Origin,
    pub sync: bool,
    pub stats: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub http: Option<Interface>,
}

impl Service for Config {
    fn service_title(&self) -> String {
        "Kaspa RPC resolver".to_string()
    }

    fn service_name(&self) -> String {
        SERVICE_NAME.to_string()
    }

    fn kind(&self) -> ServiceKind {
        ServiceKind::Resolver
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
        let proxy_kind = ProxyKind::http(8989);
        let proxy_config = ProxyConfig::new(
            format!("{} ({})", self.service_title(), self.service_name()),
            "/",
            proxy_kind,
        );
        Some(vec![proxy_config])
    }
}

impl Config {
    pub fn new(origin: Origin) -> Self {
        Self {
            enabled: false,
            kind: None,
            certs: None,
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
            git::reset(&path)?;
            git::pull(&path)?;
        } else {
            git::clone(&path, origin)?;
        }

        step.done("Resolver sources synchronized...")
    })?;

    Ok(())
}

pub fn binary(origin: &Origin) -> PathBuf {
    folder(origin).join("target/release/resolver")
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
    systemd::enable(config)?;
    systemd::start(config)?;

    Ok(())
}

pub fn certs(ctx: &Context) -> Option<Certs> {
    ctx.config
        .resolver
        .certs
        .clone()
        .or(ctx.config.nginx.certs())
}

pub fn update(ctx: &Context) -> Result<()> {
    let config = &ctx.config.resolver;

    if !config.enabled {
        return Ok(());
    }

    fetch(&config.origin)?;
    build(&config.origin)?;
    restart(ctx)?;
    Ok(())
}

pub fn uninstall(ctx: &mut Context) -> Result<()> {
    let config = &mut ctx.config.resolver;
    config.enabled = false;

    log::remark("Uninstalling resolver...")?;

    let service_name = config.service_name();

    if systemd::exists(config) {
        systemd::stop(config)?;
        systemd::disable(config)?;
        systemd::remove(config)?;
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
    let args = Vec::<String>::from(config);
    let exec_start = [binary(&config.origin).display().to_string()]
        .into_iter()
        .chain(args)
        .collect::<Vec<_>>();

    let unit_config = systemd::Config::new(config, "Kaspa Resolver", &ctx.username, exec_start, 5);

    systemd::create(unit_config)?;

    Ok(())
}

pub fn start(ctx: &Context) -> Result<()> {
    systemd::start(&ctx.config.resolver)
}

pub fn stop(ctx: &Context) -> Result<()> {
    systemd::restart(&ctx.config.resolver)
}

pub fn restart(ctx: &Context) -> Result<()> {
    step("Restarting resolver...", || {
        systemd::restart(&ctx.config.resolver)
    })
}

pub fn status(config: &Config) -> Result<String> {
    systemd::status(config)
}

pub fn check_for_updates(ctx: &Context) -> Result<()> {
    let config = &ctx.config.resolver;
    if !config.enabled() {
        return Ok(());
    }
    let mut updates = Vec::new();
    let origin = &config.origin;
    let path = folder(origin);

    let latest = git::latest_commit_hash(origin, true)?;
    let current = git::hash(path, true)?;
    if latest != current {
        log::info(format!(
            "Resolver update available ({origin}): {current} -> {latest}"
        ))?;
        updates.push((origin, current, latest));
    }

    if !updates.is_empty()
        && confirm("Update Kaspa RPC Resolver?")
            .initial_value(true)
            .interact()?
    {
        update(ctx)?;
    }

    Ok(())
}

pub fn reconfigure(_ctx: &Context, _force: bool) -> Result<()> {
    // let mut reconfigure_systemd = false;

    Ok(())
}

// ---

pub fn resolver_config_folder() -> PathBuf {
    home_folder().join(".kaspa-resolver")
}

pub fn ensure_resolver_config_folder() -> Result<()> {
    let config_folder = resolver_config_folder();
    if !config_folder.exists() {
        fs::create_dir_all(config_folder)?;
    }
    Ok(())
}

fn key_file() -> String {
    ".key".to_string()
}

fn resolver_config_file(version: usize) -> String {
    format!("resolver.{version}.bin")
}

fn load_key() -> Result<Secret> {
    Ok(Secret::from(fs::read(
        resolver_config_folder().join(key_file()),
    )?))
}

pub fn update_resolver_config_version(version: usize, key: &Secret) -> Result<()> {
    let data = reqwest::blocking::get(format!(
        "https://raw.githubusercontent.com/aspectron/kaspa-resolver/master/data/{}",
        resolver_config_file(version)
    ))?
    .bytes()?
    .to_vec();
    match chacha20poly1305::decrypt_slice(&data, key) {
        Ok(_) => {
            log::info(format!("Updating resolver config version `{version}`..."))?;
            fs::write(
                resolver_config_folder().join(resolver_config_file(version)),
                data,
            )?;
        }
        Err(err) => {
            log::error(format!(
                "Failed to decrypt resolver config version `{version}`: {err}"
            ))?;
        }
    }
    Ok(())
}

pub fn update_resolver_config() {
    let key = match load_key() {
        Ok(key) => key,
        Err(err) => {
            log::error(format!("Failed to load resolver configuration key: {err}")).ok();
            return;
        }
    };

    let mut version = 1;
    while update_resolver_config_version(version, &key).is_ok() {
        version += 1;
    }
}

pub fn init_resolver_config(ctx: &mut Context) -> Result<()> {
    if !ctx.config.resolver.enabled {
        log::warning("Resolver service is not enabled, please enable it.")?;
        return Ok(());
    }

    let mut selector = cliclack::select("Please select the type of resolver configuration");

    if let Some(selected) = &ctx.config.resolver.kind {
        selector = selector.initial_value(selected);
    }
    for kind in ResolverKind::iter() {
        selector = selector.item(kind, kind.describe(), kind.rustdoc());
    }
    let selected = selector.interact()?;

    match selected {
        ResolverKind::Public => {
            generate_key(Some(0xe311))?;
        }
        ResolverKind::Private => {
            generate_key(None)?;
        }
    }

    ctx.config.resolver.kind = Some(*selected);
    ctx.config.save()?;

    Ok(())
}

pub fn generate_key(prefix: Option<u16>) -> Result<()> {
    if resolver_config_folder().join(key_file()).exists()
        && !cliclack::confirm("Key already exists. Overwrite?").interact()?
    {
        return Ok(());
    }

    match cliclack::password("Enter password:").interact() {
        Ok(password1) => match cliclack::password("Enter password:").interact() {
            Ok(password2) => {
                if password1 != password2 {
                    return Err(Error::PasswordsDoNotMatch);
                }
                let key = argon2_sha256(password1.as_bytes(), 32)?;

                if let Some(supplied_prefix) = prefix {
                    let generated_prefix =
                        u16::from_be_bytes(key.as_bytes()[0..2].try_into().unwrap());
                    if supplied_prefix != generated_prefix {
                        return Err(Error::custom("Resolver key prefix mismatch: expected {supplied_prefix:04x} got {generated_prefix:04x}"));
                    }
                }
                fs::write(resolver_config_folder().join(key_file()), key.as_bytes())?;
                cliclack::outro("Key generated successfully")?;
                println!();
            }
            Err(_) => {
                log::error("Failed to read password")?;
            }
        },
        Err(_) => {
            log::error("Failed to read password")?;
        }
    }

    Ok(())
}
