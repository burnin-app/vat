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
use crate::git::Git;

const VAT_REPOSITORY_FILE: &str = "vat_repository.toml";


#[derive(Debug)]
pub struct PackageName{
    pub name: String,
    pub version: Option<Version>
}

impl PackageName{
    pub fn from_str(package_name: &str) -> Self{
        let parts: Vec<&str> = package_name.split('/').collect();
        if parts.len() == 1{
            Self{
                name: parts[0].to_string(),
                version: None,
            }
        }else if parts.len() == 2{
            Self{
                name: parts[0].to_string(),
                version: Some(Version::parse(parts[1]).unwrap()),
            }
        }else{
            Self{
                name: package_name.to_string(),
                version: None,
            }
        }
    }
}



#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Repository{
    pub packages: HashMap<String, PackageRegistry>,
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

    pub fn get_package(&self, package_name: &str) -> Option<&PackageRegistry>{
        self.packages.get(package_name)
    }

    pub fn get_package_by_package_name(&self, package_name: &PackageName) -> Option<&PackageRegistry>{
        self.packages.get(&package_name.name)
    }

    pub fn publish(&mut self, package: Vat, message: &str) -> RepositoryResult<()>{
        if self.package_exists(&package.package.name, &package.package.version){
            return Err(RepositoryError::PackageAlreadyExists(format!("Package {} version {} has been published", package.package.name, package.package.version)));
        }

        let git = Git::init(package.package_path.clone())?;

        let mut repo_package = RepoPackage::from_vat(package.clone());
        let package_path = self.repository_path.join(repo_package.name.clone()).join(package.package.version.to_string());
        if !package_path.exists(){
            std::fs::create_dir_all(&package_path)?;
        }

        repo_package.package_path = package_path;
        repo_package.message = Some(message.to_string());

        // link the package to the repoistiory
        self.packages.entry(repo_package.name.clone())
            .or_insert_with(|| PackageRegistry::new())
            .link_package(package.package_path.clone());

        self.packages.entry(repo_package.name.clone())
            .or_insert_with(|| PackageRegistry::new())
            .add_package(repo_package);

        let repo_package_path = self.repository_path.join(package.package.name.clone());
        git.zip_tag(package.package.version, package.package_path.clone(), repo_package_path)?;
        self.save()?;
        Ok(())
    }


    pub fn link_package(&mut self, package: Vat) -> RepositoryResult<()>{
        self.packages.entry(package.package.name.clone())
            .or_insert_with(|| PackageRegistry::new())
            .link_package(package.package_path.clone());

        self.save()?;
        Ok(())
    }



    pub fn package_exists(&self, package_name: &str, version: &Version) -> bool{
        if self.packages.contains_key(package_name){
            if self.packages[package_name].versions.contains_key(version){
                return true;
            }
        }
        false
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
        let repository = repository.read()?;

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
pub struct PackageRegistry{
    pub main_brach_path: PathBuf,
    pub versions: HashMap<Version, RepoPackage>,
}

impl PackageRegistry{
    pub fn new() -> Self{
        Self{
            main_brach_path: PathBuf::new(),
            versions: HashMap::new(),
        }
    }

    pub fn add_package(&mut self, package: RepoPackage){
        self.versions.insert(package.version.clone(), package);
    }

    pub fn link_package(&mut self, main_brach_path: PathBuf){
        self.main_brach_path = main_brach_path;
    }

    pub fn get_package_path(&self, package_name: &PackageName) -> Option<PathBuf>{
        if package_name.version.is_none(){
            return Some(self.main_brach_path.clone());
        }else{
            let version = package_name.version.as_ref().unwrap();
            if self.versions.contains_key(&version){
                return Some(self.versions[&version].package_path.clone());
            }
        }
        None
    }

}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RepoPackage{
    pub name: String,
    pub version: Version,
    pub package_path: PathBuf,
    pub repository: Option<Url>,
    pub message: Option<String>,
}

impl RepoPackage{

    pub fn from_vat(vat: Vat) -> Self{
        Self{
            name: vat.package.name,
            version: vat.package.version,
            package_path: PathBuf::from(""),
            repository: vat.package.repository,
            message: None,
        }
    }


}