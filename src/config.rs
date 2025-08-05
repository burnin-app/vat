use std::path::PathBuf;
use serde::{Serialize, Deserialize};
use dirs_next::{config_dir, document_dir};
use std::fs;
use crate::repository::Repository;

const CONFIG_FILE_NAME: &str = "vat.config";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct VatConfig{
    pub repository_path: PathBuf,   
    pub packages_path: PathBuf,
}


impl VatConfig {
    pub fn new() -> Self{
        VatConfig { repository_path: PathBuf::new(), packages_path: PathBuf::new() }
    }

    pub fn init() -> Result<Self, anyhow::Error> {
        let app_dir = VatConfig::get_app_dir();
        if app_dir.is_none(){
            return Err(anyhow::anyhow!("Filed to get app directory"));
        }

        let app_dir = app_dir.unwrap();
        if !app_dir.exists(){
            fs::create_dir_all(&app_dir)?;
        }

        let mut config = VatConfig::new();

        let config_path = app_dir.join(CONFIG_FILE_NAME);
        if !config_path.exists(){
            // default config
            let repoistory_path = app_dir.join("repository");
            let packages_path = app_dir.join("packages");

            config.repository_path = repoistory_path;
            config.packages_path = packages_path;

            config.save()?;

        }else{
            let config_str = fs::read_to_string(config_path)?;
            let config_result = toml::from_str(&config_str)?;
            config = config_result;
        }

        if !config.repo_exists(){
            println!("Creating default repository");
            fs::create_dir_all(&config.repository_path)?;

            // create a default repository
            let mut repository = Repository::new();
            repository.repository_path = config.repository_path.clone();
            repository.save()?;
        }

        if !config.packages_exists(){
            fs::create_dir_all(&config.packages_path)?;
        }

        Ok(config)
    }

    pub fn repo_exists(&self) -> bool{
        self.repository_path.exists()
    }

    pub fn packages_exists(&self) -> bool{
        self.packages_path.exists()
    }


    pub fn get_repository_path(&self) -> PathBuf {
        self.repository_path.clone()
    }

    pub fn set_repository_path(&mut self, path: PathBuf) {
        self.repository_path = path;
    }

    pub fn save(&self) -> Result<(), anyhow::Error> {
        let config_path = VatConfig::get_app_dir().unwrap().join(CONFIG_FILE_NAME);
        let config_str = toml::to_string(&self).unwrap();
        fs::write(config_path, config_str).unwrap();
        Ok(())
    }

    pub fn get_app_dir() -> Option<PathBuf> {
        let app_name = String::from("Vat");

        if cfg!(target_os = "macos"){
            config_dir().map(|path| path.join(app_name))
        }
        else if cfg!(target_os = "windows") {
            document_dir().map(|path| path.join(app_name))
        } else {
            config_dir().map(|path| path.join(app_name))
        }
    }
}