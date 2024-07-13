use crate::imports::*;
use system::System;

pub struct Context {
    pub args: Args,
    pub system: Arc<System>,
    pub username: String,
    pub config: Config,
}

impl Context {
    pub fn try_new(args: Args) -> Result<Self> {
        let username = whoami::username();
        let system = Arc::new(System::default());

        let config = match Config::load() {
            Ok(config) => config,
            Err(_) => {
                let config = Config::try_new()?;
                config.save()?;
                config
            }
        };

        Ok(Context {
            args,
            system,
            username,
            config,
        })
    }

    pub fn active_services(&self) -> Vec<ServiceDetail> {
        let mut services = kaspad::active_configs(self)
            .map(|config| config.service_detail())
            .collect::<Vec<_>>();

        if self.config.resolver.enabled {
            services.push(self.config.resolver.service_detail());
        }

        services.push(nginx::nginx_service_detail());

        services
    }

    pub fn select_active_service<C>(&self, caption: C) -> Result<ServiceDetail>
    where
        C: Display,
    {
        let mut services = kaspad::active_configs(self)
            .map(|config| config.service_detail())
            .collect::<Vec<_>>();

        if self.config.resolver.enabled {
            services.push(self.config.resolver.service_detail());
        }

        services.push(nginx::nginx_service_detail());

        let mut selector = cliclack::select(caption.to_string());
        for service in services {
            selector = selector.item(service.clone(), service, "");
        }

        Ok(selector.interact()?)
    }
}
