use crate::imports::*;

const SYSTEMD_SERVICE_PATH: &str = "/etc/systemd/system";

// #[derive(Builder)]
pub struct Config {
    pub service_name: String,
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
    pub fn new<ServiceName, Description, User>(
        service_name: ServiceName,
        description: Description,
        user: User,
        exec_start: Vec<String>,
        restart_secs: u64,
    ) -> Self
    where
        ServiceName: Display,
        Description: Display,
        User: Display,
    {
        Self {
            service_name: service_name.to_string(),
            description: description.to_string(),
            user: user.to_string(),
            exec_start: exec_start.join(" "),
            restart_secs,
        }
    }
}

pub fn enable<S: Display>(service_name: S) -> Result<()> {
    sudo!("systemctl", "enable", service_name.to_string()).run()
}

pub fn disable<S: Display>(service_name: S) -> Result<()> {
    sudo!("systemctl", "disable", service_name.to_string()).run()
}

pub fn start<S: Display>(service_name: S) -> Result<()> {
    sudo!("systemctl", "start", service_name.to_string()).run()
}

pub fn stop<S: Display>(service_name: S) -> Result<()> {
    sudo!("systemctl", "stop", service_name.to_string()).run()
}

pub fn restart<S: Display>(service_name: S) -> Result<()> {
    sudo!("systemctl", "restart", service_name.to_string()).run()
}

pub fn status<S: Display>(service_name: S) -> Result<String> {
    sudo!("systemctl", "status", service_name.to_string()).read()
}

pub fn is_active<S: Display>(service_name: S) -> Result<bool> {
    let output = sudo!("systemctl", "is-active", service_name.to_string())
        .unchecked()
        .read()?;
    Ok(output.trim() == "active")
}

pub fn is_enabled<S: Display>(service_name: S) -> Result<bool> {
    let output = sudo!("systemctl", "is-enabled", service_name.to_string())
        .unchecked()
        .read()?;
    Ok(output.trim() == "enabled")
}

pub fn is_failed<S: Display>(service_name: S) -> Result<bool> {
    let output = sudo!("systemctl", "is-failed", service_name.to_string())
        .unchecked()
        .read()?;
    Ok(output.trim() == "failed")
}

pub fn is_active_resp<S: Display>(service_name: S) -> Result<String> {
    sudo!("systemctl", "is-active", service_name.to_string())
        .unchecked()
        .read()
        .map(|resp| resp.trim().to_string())
}

pub fn is_enabled_resp<S: Display>(service_name: S) -> Result<String> {
    sudo!("systemctl", "is-enabled", service_name.to_string())
        .unchecked()
        .read()
        .map(|resp| resp.trim().to_string())
}

pub fn is_failed_resp<S: Display>(service_name: S) -> Result<String> {
    sudo!("systemctl", "is-failed", service_name.to_string())
        .unchecked()
        .read()
        .map(|resp| resp.trim().to_string())
}

pub fn reload<S: Display>(service_name: S) -> Result<()> {
    step(format!("Reloading {service_name}..."), || {
        sudo!("systemctl", "reload", service_name.to_string()).run()
    })
}

pub fn daemon_reload() -> Result<()> {
    step("Reloading systemd daemons...", || {
        sudo!("systemctl", "daemon-reload").run()
    })
}

pub fn exists(service_name: &str) -> bool {
    service_path(service_name).exists()
}

pub fn service_path<S: Display>(service_name: S) -> PathBuf {
    Path::new(SYSTEMD_SERVICE_PATH).join(format!("{service_name}.service"))
}

pub fn create(config: Config) -> Result<()> {
    let service_path = service_path(&config.service_name);
    sudo::fs::write(service_path, config.to_string())?;
    Ok(())
}

pub fn remove<S: Display>(service_name: S) -> Result<()> {
    let service_path = service_path(&service_name);
    if service_path.exists() {
        sudo::fs::remove_file(service_path)?;
    }
    Ok(())
}
