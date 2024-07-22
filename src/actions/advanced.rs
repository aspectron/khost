use crate::imports::*;

#[derive(Describe, Eq, PartialEq, Debug, Clone, Copy)]
#[caption = "Advanced menu"]
pub enum Advanced {
    #[describe("Back")]
    Back,
    #[describe("Configure Git")]
    Git,
    #[describe("Full installation")]
    Full,
    #[describe("Delete Kaspa Data folders")]
    PurgeData,
    #[describe("Uninstall Kaspa software")]
    Uninstall,
    #[describe("Toggle sudo password")]
    SudoersEntry,
    #[describe("Generate resolver key")]
    ResolverKey,
}

impl Action for Advanced {
    fn main(&self, ctx: &mut Context) -> Result<bool> {
        match self {
            Advanced::Back => Ok(false),
            Advanced::PurgeData => {
                let mut folders = HashMap::new();
                for config in ctx.config.kaspad.iter() {
                    let data_folder = config.data_folder();
                    println!("Checking folder: {:?}", data_folder.display());
                    if data_folder.exists() {
                        println!("Folder exists: {:?}", data_folder.display());
                        let info = cmd!("du", "-h", "-s", &data_folder).read()?;
                        folders.insert(info, (config.network(), data_folder));
                    } else {
                        println!("Folder does not exist: {:?}", data_folder.display());
                    }
                }

                let list = folders
                    .keys()
                    .map(|x| x.to_string())
                    .collect::<Vec<String>>();
                let mut selector =
                    cliclack::multiselect("Select Kaspa Data folders to delete (ESC to cancel)");
                for item in list.iter() {
                    selector = selector.item(item, item, "");
                }
                let selection = selector.interact().ok();
                if let Some(selection) = selection {
                    let targets = selection
                        .into_iter()
                        .map(|x| folders.get(x).unwrap())
                        .collect::<Vec<_>>();
                    println!("Selected folders: {:?}", targets);
                }
                Ok(true)
            }
            Advanced::Full => {
                actions::Bootstrap::select(ctx)?;
                Ok(true)
            }
            Advanced::Uninstall => {
                if confirm("Are you sure you want to uninstall Kaspa software?").interact()? {
                    log::step("Uninstalling Kaspa software")?;
                    resolver::uninstall(ctx)?;
                    kaspad::uninstall(ctx)?;
                    nginx::remove()?;
                    log::success("Kaspa software uninstalled successfully")?;
                }
                Ok(true)
            }
            Advanced::Git => {
                let services = ctx.managed_services();
                let list = vec![BranchChange::Kaspad]
                    .into_iter()
                    .chain(services.into_iter().map(BranchChange::Service))
                    .collect::<Vec<_>>();
                let mut selector = cliclack::select("Select service to configure Git origin");
                for item in list.iter() {
                    selector = selector.item(item, item, "");
                }
                match selector.interact() {
                    Ok(BranchChange::Kaspad) => {
                        let origin = git::create_origin("rusty-kaspa")?;
                        for config in ctx.config.kaspad.iter_mut() {
                            config.set_origin(origin.clone());
                        }
                        ctx.config.save()?;
                        log::success("Git origin updated successfully")?;
                        kaspad::update(ctx)?;
                        Ok(true)
                    }
                    Ok(BranchChange::Service(service)) => {
                        let name = service
                            .origin
                            .as_ref()
                            .expect("Service origin not set")
                            .name();
                        let origin = git::create_origin(name)?;
                        match &service.kind {
                            ServiceKind::Kaspad(network) => {
                                kaspad::find_config_by_network(ctx, network)
                                    .expect("Kaspad config not found")
                                    .set_origin(origin);
                                ctx.config.save()?;
                                log::success("Git origin updated successfully")?;
                                kaspad::update(ctx)?;
                                Ok(true)
                            }
                            ServiceKind::Resolver => {
                                ctx.config.resolver.origin = origin.clone();
                                ctx.config.save()?;
                                log::success("Git origin updated successfully")?;
                                resolver::update(ctx)?;
                                Ok(true)
                            }
                            kind => {
                                log::error(format!("Service {kind:?} not supported"))?;
                                Ok(true)
                            }
                        }
                    }
                    Err(e) => {
                        log::error(e)?;
                        Ok(true)
                    }
                }
                // Ok(true)
            }
            Advanced::SudoersEntry => {
                sudo::toggle_sudoers_entry(ctx).ok();
                Ok(true)
            }
            Advanced::ResolverKey => {
                resolver::init_resolver_config(ctx)?;
                Ok(true)
            }
        }
    }
}

#[derive(Eq, PartialEq, Debug, Clone)]
pub enum BranchChange {
    Kaspad,
    Service(ServiceDetail),
}

impl Display for BranchChange {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            BranchChange::Kaspad => write!(f, "Kaspad p2p node (all instances)"),
            BranchChange::Service(service) => write!(f, "{}", service),
        }
    }
}
