use crate::imports::*;

const SYSTEMD_SERVICE_PATH: &str = "/etc/systemd/system";

pub struct Config {
    pub service: String,
    pub description: String,
    pub user: String,
    pub exec_start: String,
    pub restart_secs: u64,
}

impl Display for Config {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        writeln!(f, "[Unit]")?;
        writeln!(f, "Description={}", self.description)?;
        writeln!(f)?;
        writeln!(f, "[Service]")?;
        writeln!(f, "User={}", self.user)?;
        writeln!(f, "ExecStart={}", self.exec_start)?;
        writeln!(f, "RestartSec={}", self.restart_secs)?;
        writeln!(f, "Restart=on-failure")?;
        writeln!(f)?;
        writeln!(f, "[Install]")?;
        writeln!(f, "WantedBy=multi-user.target")?;
        Ok(())
    }
}

impl Config {
    pub fn new<S, Description, User>(
        service: &S,
        description: Description,
        user: User,
        exec_start: Vec<String>,
        restart_secs: u64,
    ) -> Self
    where
        S: Service,
        Description: Display,
        User: Display,
    {
        Self {
            service: service.service_name(),
            description: description.to_string(),
            user: user.to_string(),
            exec_start: exec_start.join(" "),
            restart_secs,
        }
    }
}

pub fn enable<S: Service>(service: &S) -> Result<()> {
    sudo!("systemctl", "enable", service.service_name()).run()
}

pub fn disable<S: Service>(service: &S) -> Result<()> {
    sudo!("systemctl", "disable", service.service_name()).run()
}

pub fn start<S: Service>(service: &S) -> Result<()> {
    sudo!("systemctl", "start", service.service_name()).run()
}

pub fn stop<S: Service>(service: &S) -> Result<()> {
    sudo!("systemctl", "stop", service.service_name()).run()
}

pub fn restart<S: Service>(service: &S) -> Result<()> {
    sudo!("systemctl", "restart", service.service_name()).run()
}

pub fn status<S: Service>(service: &S) -> Result<String> {
    sudo!("systemctl", "status", service.service_name()).read()
}

pub fn is_active<S: Service>(service: &S) -> Result<bool> {
    let output = sudo!("systemctl", "is-active", service.service_name())
        .unchecked()
        .read()?;
    Ok(output.trim() == "active")
}

pub fn is_enabled<S: Service>(service: &S) -> Result<bool> {
    let output = sudo!("systemctl", "is-enabled", service.service_name())
        .unchecked()
        .read()?;
    Ok(output.trim() == "enabled")
}

pub fn is_failed<S: Service>(service: &S) -> Result<bool> {
    let output = sudo!("systemctl", "is-failed", service.service_name())
        .unchecked()
        .read()?;
    Ok(output.trim() == "failed")
}

pub fn is_active_resp<S: Service>(service: &S) -> Result<String> {
    sudo!("systemctl", "is-active", service.service_name())
        .unchecked()
        .read()
        .map(|resp| resp.trim().to_string())
}

pub fn is_enabled_resp<S: Service>(service: &S) -> Result<String> {
    sudo!("systemctl", "is-enabled", service.service_name())
        .unchecked()
        .read()
        .map(|resp| resp.trim().to_string())
}

pub fn unit_state<S: Display>(service_name: S) -> std::result::Result<String, String> {
    let enabled = sudo!("systemctl", "is-enabled", service_name.to_string())
        .unchecked()
        .read()
        .map(|resp| resp.trim().to_string())
        .map_err(|err| err.to_string())?;
    let active = sudo!("systemctl", "is-active", service_name.to_string())
        .unchecked()
        .read()
        .map(|resp| resp.trim().to_string())
        .map_err(|err| err.to_string())?;

    if enabled == "enabled" && active == "active" {
        Ok("enabled+active".to_string())
    } else {
        Err(format!("{}+{}", enabled, active))
    }
}

pub fn is_failed_resp<S: Service>(service: &S) -> Result<String> {
    sudo!("systemctl", "is-failed", service.service_name())
        .unchecked()
        .read()
        .map(|resp| resp.trim().to_string())
}

pub fn reload<S: Service>(service: &S) -> Result<()> {
    step(format!("Reloading {}...", service.service_name()), || {
        sudo!("systemctl", "reload", service.service_name()).run()
    })
}

pub fn daemon_reload() -> Result<()> {
    sudo!("systemctl", "daemon-reload").run()
}

pub fn exists<S: Service>(service: &S) -> bool {
    service_path(service.service_name().as_str()).exists()
}

pub fn service_path(service_name: &str) -> PathBuf {
    Path::new(SYSTEMD_SERVICE_PATH).join(format!("{service_name}.service"))
}

pub fn create(config: Config) -> Result<()> {
    let service_path = service_path(&config.service);
    sudo::fs::write(service_path, config.to_string())?;
    Ok(())
}

pub fn remove<S: Service>(service: &S) -> Result<()> {
    let service_path = service_path(&service.service_name());
    if service_path.exists() {
        sudo::fs::remove_file(service_path)?;
    }
    Ok(())
}
