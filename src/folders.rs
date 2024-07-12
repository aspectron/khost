use crate::imports::*;

cfg_if! {
    if #[cfg(target_os = "linux")] {
        const DATA_FOLDER_NAME: &str = ".khost";
        const ROOT_FOLDER_NAME: &str = "kaspa";
        const TEMP_FOLDER_NAME: &str = ".temp";
    } else {
        const DATA_FOLDER_NAME: &str = "00-khost-dev";
        const ROOT_FOLDER_NAME: &str = "00-kaspa-dev";
        const TEMP_FOLDER_NAME: &str = "00-temp-dev";
    }
}

/// KHost data folder (e.g. ~/.khost) used for configuration storage
pub fn data_folder() -> PathBuf {
    static DATA_FOLDER: OnceLock<PathBuf> = OnceLock::new();

    DATA_FOLDER
        .get_or_init(|| {
            let data_folder = home_folder().join(DATA_FOLDER_NAME);
            fs::create_dir_all(&data_folder).expect("Unable to create ~/.khost data folder");
            data_folder
        })
        .clone()
}

/// User home folder
pub fn home_folder() -> PathBuf {
    static HOME_FOLDER: OnceLock<PathBuf> = OnceLock::new();

    HOME_FOLDER
        .get_or_init(|| dirs::home_dir().expect("Home folder not found"))
        .clone()
}

/// Folder that contains all installed applications
pub fn root_folder() -> PathBuf {
    static ROOT_FOLDER: OnceLock<PathBuf> = OnceLock::new();

    ROOT_FOLDER
        .get_or_init(|| {
            let root_folder = home_folder().join(ROOT_FOLDER_NAME);
            fs::create_dir_all(&root_folder).expect("Unable to create ~/.khost data folder");
            root_folder
        })
        .clone()
}

pub fn temp_folder() -> PathBuf {
    static TEMP_FOLDER: OnceLock<PathBuf> = OnceLock::new();

    TEMP_FOLDER
        .get_or_init(|| {
            let temp_folder = home_folder().join(TEMP_FOLDER_NAME);
            fs::create_dir_all(&temp_folder).expect("Unable to create ~/.khost data folder");
            temp_folder
        })
        .clone()
}
