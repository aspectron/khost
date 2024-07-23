use crate::imports::*;
use crate::system::System;

type ServiceStateVec = Vec<(ServiceDetail, std::result::Result<String, String>)>;

pub struct Status {
    pub ip: Option<String>,
    pub system: Arc<System>,
    pub services: ServiceStateVec,
}

impl Display for Status {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let mut rows: Vec<Content> = self.system.as_ref().into();
        rows.push(Content::separator());
        rows.push(Content::field(
            "System id:",
            self.system
                .system_id
                .as_ref()
                .map(|id| style(format!("{id:016x}")).cyan().bright())
                .unwrap_or(style("N/A".to_string()).red().bright()),
        ));
        rows.push(Content::field(
            "Public ip:",
            self.ip
                .as_deref()
                .map(|ip| style(ip).cyan().bright())
                .unwrap_or(style("N/A").red().bright()),
        ));
        rows.push(Content::separator());
        for (service, status) in self.services.iter() {
            let content = match status {
                Ok(status) => Content::field(service, style(status).green().bright()),
                Err(status) => Content::field(service, style(status).red().bright()),
            };
            rows.push(content);
            if let Some(origin) = &service.origin {
                rows.push(Content::field("", origin));
            }
        }

        writeln!(f, "{}", content(rows))?;
        Ok(())
    }
}

pub fn detect(ctx: &Context) -> Status {
    let ip = ip::blocking::public().ok();
    let system = ctx.system.clone();

    let services = ctx
        .active_services()
        .into_iter()
        .map(|service| {
            if service.kind == ServiceKind::Nginx {
                match systemd::is_enabled(&service.name) {
                    Ok(true) => {
                        let status = systemd::unit_state(&service.name);
                        (service, status)
                    }
                    Ok(false) => (service, Err("n/a".to_string())),
                    Err(e) => (service, Err(e.to_string())),
                }
            } else {
                let status = systemd::unit_state(&service.name);
                (service, status)
            }
        })
        .collect();

    Status {
        ip,
        system,
        services,
        // errors,
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

pub fn conflicts(ctx: &Context, _status: &Status) {
    use sysinfo::*;

    let mut system = System::new();
    system.refresh_processes();

    let kaspad_paths = system
        .processes()
        .values()
        .filter_map(|proc| {
            proc.exe().and_then(|path| {
                let path_str = path.display().to_string();
                (["kaspad", "kaspa-ng", "kaspa-resolver", "sparkled"]
                    .iter()
                    .any(|k| path_str.contains(k)))
                .then_some(path)
            })
        })
        .collect::<HashSet<_>>();

    let mut abort = false;
    let root_folder = root_folder();
    for path in kaspad_paths {
        if !path.starts_with(&root_folder) {
            log::warning(format!(
                "{} `{}`\n{}",
                style("Detected conflicting process at:").red().bright(),
                path.display(),
                style("Please make sure no other process instances are running on this system.")
                    .yellow()
            ))
            .ok();
            abort = true;
        }
    }

    if abort {
        outro("Unable to continue until conflicts are resolved.").ok();
        println!();
        std::process::exit(1);
    }

    let networks = ctx
        .config
        .kaspad
        .iter()
        .map(Service::enabled)
        .filter(|enabled| *enabled)
        .collect::<Vec<_>>()
        .len();

    if !kaspad::supports_multiple_networks(ctx, networks) {
        log::error(format!(
            "Detected RAM of {} is insufficient for {} networks!",
            as_gb(ctx.system.total_memory as f64, false, false),
            networks
        ))
        .ok();
    }

    rust::check().ok();
}
