use crate::imports::*;

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize, Hash)]
pub enum ServiceKind {
    Kaspad(Network),
    Resolver,
    Nginx,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct ServiceDetail {
    pub caption: String,
    pub name: String,
    pub kind: ServiceKind,
    pub origin: Option<Origin>,
    pub enabled: bool,
    pub managed: bool,
}

impl ServiceDetail {
    pub fn new<C, N>(
        caption: C,
        name: N,
        kind: ServiceKind,
        origin: Option<Origin>,
        enabled: bool,
        managed: bool,
    ) -> Self
    where
        C: Display,
        N: Display,
    {
        Self {
            caption: caption.to_string(),
            name: name.to_string(),
            kind,
            origin,
            enabled,
            managed,
        }
    }
}

impl Display for ServiceDetail {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{} ({})", self.caption, self.name)
    }
}

pub trait Service {
    fn service_title(&self) -> String;
    fn service_name(&self) -> String;
    fn kind(&self) -> ServiceKind;
    fn origin(&self) -> Option<Origin>;
    fn enabled(&self) -> bool;
    fn managed(&self) -> bool;
    fn proxy_config(&self, ctx: &Context) -> Option<Vec<ProxyConfig>>;

    fn service_detail(&self) -> ServiceDetail {
        ServiceDetail::new(
            self.service_title(),
            self.service_name(),
            self.kind(),
            self.origin(),
            self.enabled(),
            self.managed(),
        )
    }
}

pub fn enable_services(ctx: &mut Context, services: Vec<ServiceDetail>) -> Result<()> {
    let mut kaspad_networks = Vec::new();

    for service in services {
        match service.kind {
            ServiceKind::Kaspad(network) => kaspad_networks.push(network),
            ServiceKind::Resolver => {
                ctx.config.resolver.enabled = true;
                ctx.config.save()?;

                if !resolver::is_installed(ctx) {
                    resolver::install(ctx)?;
                }

                if !systemd::is_enabled(&ctx.config.resolver)? {
                    systemd::enable(&ctx.config.resolver)?;
                }
            }
            _ => {}
        }
    }

    kaspad::configure_networks(ctx, kaspad_networks)?;
    nginx::reconfigure(ctx)?;

    Ok(())
}
