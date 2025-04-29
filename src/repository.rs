use url::Url;
use std::path::PathBuf;
use semver::Version;
use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use fs2::FileExt;
use std::fs::OpenOptions;
use std::io::Write;

use crate::Vat;
use crate::config::VatConfig;
use crate::errors::{RepositoryError, RepositoryResult};

const VAT_REPOSITORY_FILE: &str = "vat_repository.toml";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Repository{
    pub packages: HashMap<String, Vec<RepoPackage>>,
    #[serde(skip)]
    pub repository_path: PathBuf,
}

impl Repository{
    pub fn new() -> Repository{
        Repository{
            packages: HashMap::new(),
            repository_path: PathBuf::new(),
        }
    }

    pub fn load() -> RepositoryResult<Repository>{
        let config_result = VatConfig::init();
        if config_result.is_err(){
            return Err(RepositoryError::ConfigError(config_result.err().unwrap().to_string()));
        }
        let config = config_result.unwrap();

        let mut repository = Repository::new();
        repository.repository_path = config.repository_path;

        // NOT SURE IF THIS IS THE BEST WAY TO HANDLE THIS
        // TODO: Find a better way to handle this
        let repository_read_result = repository.read();
        if repository_read_result.is_err(){
            repository.save()?;
        }

        Ok(repository)
    }


    pub fn read(&self) -> RepositoryResult<Repository> {
        let vat_repository_path = self.repository_path.join(VAT_REPOSITORY_FILE);
        if !vat_repository_path.exists() {
            return Err(RepositoryError::RepositoryNotFound("Cannot find vat_repository.toml".to_string()));
        }
        
        let file = OpenOptions::new()
            .read(true)
            .open(&vat_repository_path)?;
        
        FileExt::lock_shared(&file)?;
        
        let toml_string = std::fs::read_to_string(vat_repository_path)?;
        let mut repository: Repository = toml::from_str(&toml_string)?;
        repository.repository_path = self.repository_path.clone();

        FileExt::unlock(&file)?;
        
        Ok(repository)
    }


    pub fn save(&self) -> RepositoryResult<Self> {
        let vat_repository_path = self.repository_path.join(VAT_REPOSITORY_FILE);
        let mut file = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(&vat_repository_path)?;
        
        FileExt::lock_exclusive(&file)?;
        
        let toml_string = toml::to_string(self)?;
        file.write_all(toml_string.as_bytes())?;
        
        FileExt::unlock(&file)?;
        
        Ok(self.clone())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RepoPackage{
    pub name: String,
    pub version: Version,
    pub package_path: PathBuf,
    pub main_path: PathBuf,
    pub repository: Option<Url>,
}

impl RepoPackage{

    pub fn from_vat(vat: Vat) -> Self{
        Self{
            name: vat.package.name,
            version: vat.package.version,
            package_path: PathBuf::from(""),
            main_path: vat.package_path,
            repository: vat.package.repository,
        }
    }


}