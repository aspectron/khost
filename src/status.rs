use crate::imports::*;
use crate::system::System;

type ServiceStateVec = Vec<(ServiceDetail, std::result::Result<String, String>)>;

pub struct Status {
    pub ip: Option<String>,
    pub system: Arc<System>,
    // pub services: Vec<(String, String)>,
    // pub errors: Vec<(String, String)>,
    pub services: ServiceStateVec,
}

impl Display for Status {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let mut rows: Vec<Content> = self.system.as_ref().into();
        rows.push(Content::separator());
        rows.push(Content::field(
            "Public IP:",
            self.ip
                .as_deref()
                .map(|ip| style(ip).cyan())
                .unwrap_or(style("N/A").red()),
        ));
        rows.push(Content::separator());
        rows.extend(
            self.services
                .iter()
                .map(|(service, status)| match status {
                    Ok(status) => Content::field(service, style(status).green()),
                    Err(status) => Content::field(service, style(status).red()),
                })
                .collect::<Vec<_>>(),
        );

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
            let status = systemd::unit_state(service_name(&service));
            (service, status)
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

pub fn conflicts(_ctx: &Context, _status: &Status) {
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
                .then_some((proc, path))
            })
        })
        .collect::<Vec<_>>();

    let root_folder = root_folder();
    for (_proc, path) in kaspad_paths {
        if !path.starts_with(&root_folder) {
            log::error(format!(
                "{} `{}`\n{}",
                style("Detected unknown process at:").red(),
                path.display(),
                style("Please make sure no other process instances are running on this system.")
                    .yellow()
            ))
            .ok();
        }
    }

    rust::check().ok();
}
