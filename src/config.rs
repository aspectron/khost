use crate::imports::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    // Bootstrap was executed
    pub bootstrap: bool,
    pub public: bool,
    pub fqdn: Option<String>,
    pub ip: Option<String>,
    pub kaspad: Vec<kaspad::Config>,
    pub resolver: resolver::Config,
    // pub ssl : bool,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            bootstrap: false,
            public: true,
            fqdn: None,
            ip: None,
            kaspad: vec![kaspad::Config::default()],
            resolver: resolver::Config::default(),
            // ssl: false,
        }
    }
}

impl Config {
    pub fn load() -> Result<Self> {
        let config_path = data_folder().join("config.toml");
        if !config_path.exists() {
            return Err(Error::custom("Config file not found"));
        }
        Ok(serde_json::from_str(&fs::read_to_string(config_path)?)?)
    }

    pub fn save(&self) -> Result<()> {
        let config_path = data_folder().join("config.toml");
        fs::write(config_path, serde_json::to_string_pretty(&self)?)?;
        Ok(())
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
        if let Some(ip) = ctx.config.ip.as_ref() {
            cliclack::note(
                "Public Node Network",
                format!(
                    r#"
Thank you for contributing to the Kaspa ecosystem!

Your public IPv4 address is: {}

Please reach out to one of the public node maintainers
on Kaspa Discord to have your node registered.
"#,
                    ::console::style(ip).cyan()
                ),
            )?;
        } else {
            log::error("Unable to detect your public IPv4 address.\nPlease resolve this problem before continuing.")?;
            return Err(Error::custom("Unable to detect public ip :("));
        }
    } else {
        let fqdn = fqdn("Enter the fully qualified domain name (FQDN):")?;
        ctx.config.fqdn = Some(fqdn);
    }
    ctx.config.save()?;
    Ok(())
}
