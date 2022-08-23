pub mod pixiv;
pub mod reddit;
pub mod twitter;
pub mod urls;
use crate::error::Result;
use directories::ProjectDirs;
use std::{fs, path::PathBuf};

pub fn get_project_dirs() -> ProjectDirs {
    ProjectDirs::from("net", "trivernis", "hydrus-utils")
        .expect("Could not create application directories")
}

pub fn get_config_dir() -> Result<PathBuf> {
    let dirs = get_project_dirs();
    let config_dir = dirs.config_dir();

    if !config_dir.exists() {
        fs::create_dir_all(config_dir)?;
    }

    Ok(PathBuf::from(config_dir))
}
