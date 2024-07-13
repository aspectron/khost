use crate::imports::*;

#[derive(Describe, Eq, PartialEq, Debug, Clone, Copy)]
#[caption = "Advanced menu"]
pub enum Advanced {
    /// Go back to the previous menu
    #[describe("Back")]
    Back,
    #[describe("Delete Kaspa Data folders")]
    PurgeData,
    #[describe("Full installation")]
    Full,
    #[describe("Uninstall Kaspa software")]
    Uninstall,
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
                        // folders.push((config.network(),data_folder,info));
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
                // list.iter().for_each(|x| {selector = selector.item(x,x,"");});
                let selection = selector.interact().ok();
                if let Some(selection) = selection {
                    let targets = selection
                        .into_iter()
                        .map(|x| folders.get(x).unwrap())
                        .collect::<Vec<_>>();
                    println!("Selected folders: {:?}", targets);
                }
                Ok(true)

                // if confirm("Are you sure you want to delete the Kaspa Data folder?").interact()? {
                //     kaspad::stop_all(ctx)?;
                //     kaspad::purge_data_folder_all(ctx)?;
                //     // log::step("Deleting Kaspa Data folder")?;
                //     // resolver::purge_data()?;
                //     // kaspad::purge_data(ctx)?;
                //     // log::success("Kaspa Data folder deleted successfully")?;
                // }
            }
            Advanced::Full => {
                actions::Bootstrap::select(ctx)?;
                Ok(true)
            }
            Advanced::Uninstall => {
                if confirm("Are you sure you want to uninstall Kaspa software?").interact()? {
                    log::step("Uninstalling Kaspa software")?;
                    resolver::uninstall()?;
                    kaspad::uninstall(ctx)?;
                    log::success("Kaspa software uninstalled successfully")?;
                }
                Ok(true)
            }
        }
    }
}
