use crate::imports::*;

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize, Hash)]
pub enum ServiceKind {
    Kaspad(Network),
    Resolver,
    Nginx,
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize, Hash)]
pub struct ServiceDetail {
    pub caption: String,
    pub name: String,
    pub kind: ServiceKind,
    pub enabled: bool,
    pub managed: bool,
}

impl ServiceDetail {
    pub fn new<C, N>(caption: C, name: N, kind: ServiceKind, enabled: bool, managed: bool) -> Self
    where
        C: Display,
        N: Display,
    {
        Self {
            caption: caption.to_string(),
            name: name.to_string(),
            kind,
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

impl Service for ServiceDetail {
    fn service_detail(&self) -> ServiceDetail {
        self.clone()
    }
}

pub trait Service {
    fn service_detail(&self) -> ServiceDetail;
}

pub fn service_name<S: Service>(service: &S) -> String {
    service.service_detail().name
}

pub fn service_detail<S: Service>(service: &S) -> ServiceDetail {
    service.service_detail()
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

                if !systemd::is_enabled(service_name(&ctx.config.resolver))? {
                    systemd::enable(service_name(&ctx.config.resolver))?;
                }
            }
            _ => {}
        }
    }

    kaspad::configure_networks(ctx, kaspad_networks)?;

    Ok(())
}
