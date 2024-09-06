use crate::imports::*;

const CONFIG_VERSION: u64 = 2;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub version: u64,
    // Bootstrap was executed
    pub bootstrap: bool,
    pub disable_sudo_prompt: bool,
    pub public: bool,
    pub fqdn: Option<Vec<String>>,
    pub ip: Option<String>,
    pub nginx: nginx::Config,
    pub kaspad: Vec<kaspad::Config>,
    pub resolver: resolver::Config,
}

impl Config {
    pub fn try_new() -> Result<Self> {
        let origin = Origin::try_new("https://github.com/aspectron/kaspa-resolver", None)?;
        let resolver = resolver::Config::new(origin)
            .with_stats()
            .with_local_interface(8989);

        let origin = Origin::try_new("https://github.com/aspectron/rusty-kaspa", Some("pnn-v1"))?;
        let kaspad = Network::into_iter()
            .map(|network| kaspad::Config::new(origin.clone(), network))
            .collect::<Vec<_>>();

        let nginx = nginx::Config::default();

        Ok(Config {
            version: CONFIG_VERSION,
            bootstrap: false,
            disable_sudo_prompt: false,
            public: true,
            fqdn: None,
            ip: None,
            nginx,
            kaspad,
            resolver,
        })
    }
}

impl Config {
    pub fn load() -> Result<Self> {
        let config_path = data_folder().join("config.json");
        if !config_path.exists() {
            return Err(Error::custom("Config file not found"));
        }
        let mut config: Config = serde_json::from_str(&fs::read_to_string(config_path)?)?;
        
        let mut update = false;
        // Migrate old config
        if config.version < 1 {
            config.kaspad.iter_mut().for_each(|config| {
                if let Some(branch) = config.origin_mut().branch_mut() {
                    if branch == "omega" {
                        *branch = "pnn-v1".to_string();
                        update = true;
                    }
                }
            });
        }

        if update {
            config.save()?;
        }

        Ok(config)
    }

    pub fn save(&self) -> Result<()> {
        let config_path = data_folder().join("config.json");
        fs::write(config_path, serde_json::to_string_pretty(&self)?)?;
        Ok(())
    }

    pub fn reset() {
        let path = data_folder().join("config.json");
        if let Err(err) = fs::remove_file(&path) {
            let _ = log::error(format!(
                "Failed to reset config file: {}\n{err}",
                path.display()
            ));
        }
    }
}

pub fn fqdn<S: Display>(prompt: S) -> Result<String> {
    match cliclack::input(prompt)
        .validate(|input: &String| {
            if input.is_empty() {
                Err("Please enter a valid domain name".to_string())
            } else if let Err(err) = addr::parse_domain_name(input) {
                Err(err.to_string())
            } else {
                Ok(())
            }
        })
        .interact::<String>()
    {
        Ok(fqdn) => Ok(fqdn.to_string()),
        Err(e) => Err(e.into()),
    }
}

pub fn public_network(ctx: &mut Context) -> Result<()> {
    ctx.config.public =
        confirm("Would you like this node to join the Kaspa public node network?").interact()?;
    if ctx.config.public {
        if let (Some(ip), Some(id)) = (ctx.config.ip.as_ref(), ctx.system.system_id.as_ref()) {
            cliclack::note(
                "Public Node Network",
                format!(
                    r#"
Thank you for contributing to the Kaspa ecosystem!

          Your system id is: {}
Your public IPv4 address is: {}

Please register your system via the following form:
-> https://forms.gle/mWemBbwNEjXsFC5F7
Please reach out to one of the public node maintainers
on Telegram or Kaspa Discord once you have submitted
the information using this form.
"#,
                    ::console::style(format!("{id:016x}")).yellow(),
                    ::console::style(ip).cyan()
                ),
            )?;
        } else {
            log::error("Unable to detect your public IPv4 address.\nPlease resolve this problem before continuing.")?;
            return Err(Error::custom("Unable to detect public ip :("));
        }
    } else {
        // TODO: validate space-separated fqdns
        let fqdns = fqdn("Enter fully qualified domain names (FQDN):")?;
        ctx.config.fqdn = Some(fqdns.split_whitespace().map(String::from).collect());
    }
    ctx.config.save()?;
    Ok(())
}
