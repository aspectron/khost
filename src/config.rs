use crate::imports::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    // Bootstrap was executed
    pub bootstrap: bool,
    pub ip: Option<String>,
    pub kaspad: Vec<kaspad::Config>,
    pub resolver: Option<resolver::Config>,
    // pub ssl : bool,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            bootstrap: false,
            ip: None,
            kaspad: vec![kaspad::Config::default()],
            resolver: Some(resolver::Config::default()),
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

pub fn fqdn() -> Result<String> {
    match cliclack::input("Enter the fully qualified domain name (FQDN)")
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
        Ok(fqdn) => {
            Ok(fqdn.to_string())
            // Ok(())
        }
        Err(e) => {
            Err(e.into())
            // log::error(&e).ok();
            // std::process::exit(1);
        }
    }
}
