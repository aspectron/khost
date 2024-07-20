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

    pub fn proxy_configs(&self) -> Vec<ProxyConfig> {
        let mut services = self
            .config
            .kaspad
            .iter()
            .map(|config| config.proxy_config())
            .collect::<Vec<_>>();
        services.push(self.config.resolver.proxy_config());
        services.into_iter().flatten().flatten().collect::<Vec<_>>()
    }

    pub fn services(&self) -> Vec<ServiceDetail> {
        let mut services = self
            .config
            .kaspad
            .iter()
            .map(|config| config.service_detail())
            .collect::<Vec<_>>();
        services.push(self.config.resolver.service_detail());
        services.push(nginx::nginx_service_detail());
        services
    }

    pub fn active_services(&self) -> Vec<ServiceDetail> {
        self.services()
            .iter()
            .filter(|service| service.enabled)
            .cloned()
            .collect::<Vec<_>>()
    }

    pub fn managed_services(&self) -> Vec<ServiceDetail> {
        self.services()
            .iter()
            .filter(|service| service.managed)
            .cloned()
            .collect::<Vec<_>>()
    }

    pub fn managed_active_services(&self) -> Vec<ServiceDetail> {
        self.services()
            .iter()
            .filter(|service| service.enabled && service.managed)
            .cloned()
            .collect::<Vec<_>>()
    }

    pub fn select_service<C>(&self, caption: C) -> Result<ServiceDetail>
    where
        C: Display,
    {
        let services = self.services();
        let mut selector = cliclack::select(caption.to_string());
        for service in services {
            selector = selector.item(service.clone(), service, "");
        }

        Ok(selector.interact()?)
    }

    pub fn select_active_service<C>(&self, caption: C) -> Result<ServiceDetail>
    where
        C: Display,
    {
        let services = self.active_services();
        let mut selector = cliclack::select(caption.to_string());
        for service in services {
            selector = selector.item(service.clone(), service, "");
        }

        Ok(selector.interact()?)
    }
}
