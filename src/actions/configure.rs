use std::iter::once;

use crate::imports::*;
use nginx::prelude::*;

#[derive(Describe, Eq, PartialEq, Debug, Clone, Copy)]
#[caption = "Configure"]
pub enum Configure {
    /// Go back to the previous menu
    #[describe("Back")]
    Back,
    // #[describe("Fail")]
    // Fail,
    // #[describe("Verbose mode")]
    // Verbose,
    #[describe("Enable / Disable services")]
    Enable,
    #[describe("Configure SSL certificates")]
    Tls,
    #[describe("Rebuild configuration")]
    Rebuild,
    #[describe("View configuration files")]
    View,
}

impl Action for Configure {
    fn main(&self, ctx: &mut Context) -> Result<bool> {
        match self {
            // Configure::Fail => {
            //     cmd!("bash", "fail").run()?;
            // }
            Configure::Back => Ok(false),
            // Configure::Verbose => {
            //     if confirm("Enable verbose mode?")? {
            //         ctx.config.verbose = true;
            //         ctx.config.save()?;
            //     }
            // }
            Configure::Enable => {
                let services = ctx.managed_services();
                let active = ctx.managed_active_services();
                let mut selector =
                    cliclack::multiselect("Select services to enable (ESC to cancel)")
                        .initial_values(active);
                for service in services {
                    selector = selector.item(service.clone(), service, "");
                }
                match selector.interact() {
                    Ok(services) => {
                        enable_services(ctx, services)?;
                    }
                    Err(_) => {
                        println!();
                    }
                }
                Ok(true)
            }
            Configure::Tls => {
                let mut reconfigure = false;

                let certs = ctx.config.nginx.certs();
                if let Some(certs) = certs.as_ref() {
                    log::info(format!(
                        "SSL certificates are currently configured at\n{}\n{}",
                        certs.crt, certs.key
                    ))?;

                    if confirm("Disable SSL certificates?")
                        .initial_value(false)
                        .interact()?
                    {
                        ctx.config.nginx.disable_certs();
                        ctx.config.save()?;
                        log::info("SSL certificates disabled")?;
                        reconfigure = true;
                    }
                } else {
                    log::info("No SSL certificates are currently configured")?;

                    if confirm("Enable SSL certificates?")
                        .initial_value(false)
                        .interact()?
                    {
                        let located = [data_folder(), home_folder().join("certs")]
                            .into_iter()
                            .filter_map(|folder| {
                                let key = folder.join("server.key");
                                let crt = folder.join("server.crt");
                                if key.is_file()
                                    && crt.is_file()
                                    && tls::load_private_key(key.to_str().unwrap()).is_ok()
                                    && tls::load_certs(crt.to_str().unwrap()).is_ok()
                                {
                                    Some(folder)
                                } else {
                                    None
                                }
                            })
                            .collect::<Vec<_>>();

                        if located.is_empty() {
                            log::error("No SSL certificates found in data or home folder")?;
                        } else {
                            let mut selector = cliclack::select(
                                "Select a folder containing existing SSL certificates",
                            );
                            for folder in located.iter() {
                                selector =
                                    selector.item(Some(folder.clone()), folder.display(), "");
                            }
                            selector = selector.item(None, "Specify custom certificate files", "");
                            let selected = selector.interact()?;

                            match selected {
                                Some(folder) => {
                                    let key = folder.join("server.key");
                                    let crt = folder.join("server.crt");
                                    let certs = Certs::new(key, crt);
                                    ctx.config.nginx.enable_certs(certs);
                                    ctx.config.save()?;
                                    log::info("SSL certificates enabled")?;
                                }
                                None => {
                                    let key = ask_file_path(
                                        "Enter path to certificate key file (*.key)",
                                        tls::load_private_key,
                                    )?;
                                    let crt = ask_file_path(
                                        "Enter path to certificate file (*.crt)",
                                        tls::load_certs,
                                    )?;

                                    let certs = Certs::new(key, crt);
                                    ctx.config.nginx.enable_certs(certs);
                                    ctx.config.save()?;
                                    log::info("SSL certificates enabled")?;
                                }
                            }
                            reconfigure = true;
                        }
                    }
                }

                if reconfigure {
                    nginx::reconfigure(ctx)?;
                }

                Ok(true)
            }
            Configure::View => {
                let configs = ctx
                    .managed_active_services()
                    .into_iter()
                    .map(|detail| systemd::service_path(detail.name.as_str()))
                    .chain(once(nginx::config_filename()))
                    .collect::<Vec<_>>();

                configs.iter().for_each(|path| {
                    if path.exists() {
                        log::info(format!(
                            "{}\n\n{}\n",
                            style(path.display().to_string()).cyan(),
                            fs::read_to_string(path).unwrap_or_default()
                        ))
                        .unwrap();
                    } else {
                        log::error(format!("File not found\n{}", path.display())).unwrap();
                    }
                });

                Ok(true)
            }
            Configure::Rebuild => {
                kaspad::reconfigure(ctx, true)?;
                resolver::reconfigure(ctx, true)?;
                nginx::reconfigure(ctx)?;
                Ok(true)
            }
        }
    }
}

pub fn ask_file_path<S, F, T>(prompt: S, f: F) -> Result<String>
where
    S: Display,
    F: Fn(&str) -> Result<T> + 'static,
{
    let validator = move |input: &String| {
        let path = input.replace('~', home_folder().to_str().unwrap());

        if path.is_empty() {
            Err("Please enter a valid file path".to_string())
        } else if !PathBuf::from(&path).is_file() {
            Err(format!("File not found: {path}"))
        } else {
            f(path.as_str()).map_err(|err| err.to_string())?;
            Ok(())
        }
    };

    match cliclack::input(prompt)
        .validate(validator)
        .interact::<String>()
    {
        Ok(path) => Ok(path.to_string()),
        Err(e) => Err(e.into()),
    }
}
