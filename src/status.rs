use crate::imports::*;
use crate::system::System;

pub struct Status {
    pub ip: Option<String>,
    pub system: Arc<System>,
    pub services: Vec<(String, String)>,
    pub errors: Vec<(String, String)>,
}

impl Display for Status {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let mut rows: Vec<Content> = self.system.as_ref().into();
        rows.push(Content::separator());
        rows.push(Content::field(
            "Public IP:",
            self.ip.as_deref().unwrap_or("N/A"),
        ));
        rows.push(Content::separator());
        for (service, status) in &self.services {
            rows.push(Content::field(service, status));
        }

        if !self.errors.is_empty() {
            rows.push(Content::separator());
            for (service, status) in &self.errors {
                rows.push(Content::field(
                    ::console::style(service).red(),
                    ::console::style(status).red(),
                ));
            }
        }

        writeln!(f, "{}", content(rows))?;
        Ok(())
    }
}

pub fn detect(ctx: &Context) -> Status {
    let ip = ip::blocking::public().ok();
    let system = ctx.system.clone();

    let mut services = vec![];
    let nginx = systemd::is_enabled_resp("nginx").unwrap_or("error".to_string());
    services.push(("nginx".to_string(), nginx));
    let resolver = systemd::is_enabled_resp(resolver::SERVICE_NAME).unwrap_or("error".to_string());
    services.push(("resolver".to_string(), resolver));

    for config in kaspad::active_configs(ctx) {
        let service_name = config.service_name();
        let systemd_unit_enabled =
            systemd::is_enabled_resp(&service_name).unwrap_or("error".to_string());
        let systemd_unit_active =
            systemd::is_active_resp(&service_name).unwrap_or("error".to_string());
        services.push((
            service_name,
            format!("{systemd_unit_enabled}+{systemd_unit_active}"),
        ));
    }

    let mut errors = vec![];
    for config in kaspad::active_configs(ctx) {
        let service_name = config.service_name();
        let systemd_unit_enabled =
            systemd::is_enabled_resp(&service_name).unwrap_or("error".to_string());
        let systemd_unit_active =
            systemd::is_active_resp(&service_name).unwrap_or("error".to_string());
        if systemd_unit_active == "active" {
            errors.push((
                service_name,
                format!("{systemd_unit_enabled}+{systemd_unit_active}"),
            ));
        }
    }

    Status {
        ip,
        system,
        services,
        errors,
    }
}

pub enum Conflict {
    Warning(String),
    Error(String),
}

impl Conflict {
    pub fn warning(msg: impl Display) -> Self {
        Conflict::Warning(msg.to_string())
    }

    pub fn error(msg: impl Display) -> Self {
        Conflict::Error(msg.to_string())
    }

    pub fn render(&self) -> Result<()> {
        match self {
            Conflict::Warning(msg) => log::warning(msg)?,
            Conflict::Error(msg) => log::error(msg)?,
        }
        Ok(())
    }
}

#[allow(clippy::vec_init_then_push)]
pub fn conflicts(_ctx: &Context, _status: &Status) -> Option<Vec<String>> {
    let mut list = vec![];

    list.push("Kaspad is not running".to_string());
    (!list.is_empty()).then_some(list)
}
