use url::Url;
use std::path::PathBuf;
use semver::Version;
use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use fs2::FileExt;
use std::fs::OpenOptions;
use std::io::Write;

use crate::console::Console;
use crate::Vat;
use crate::Stack;
use crate::config::VatConfig;
use crate::errors::{RepositoryError, RepositoryResult};
use crate::git::Git;

const VAT_REPOSITORY_FILE: &str = "vat_repository.toml";


#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PackageName{
    pub name: String,
    pub version: PackageVersion,
    pub active: bool
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum PackageVersion{
    Version(Version),
    Latest,
    Main
}

impl std::fmt::Display for PackageVersion{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self{
            PackageVersion::Version(version) => write!(f, "{}", version),
            PackageVersion::Latest => write!(f, "latest"),
            PackageVersion::Main => write!(f, "main"),
        }
    }
}

impl PackageName{
    pub fn from_str(package_name: &str) -> Self{
        let package_name = package_name.replace(",", "");
        let parts: Vec<&str> = package_name.split('/').collect();
        if parts.len() == 1{
            Self{
                name: parts[0].to_string(),
                version: PackageVersion::Main,
                active: true,
            }
        }else if parts.len() == 2{
            let part_version = parts[1];
            if part_version == "latest"{
                Self{
                    name: parts[0].to_string(),
                    version: PackageVersion::Latest,
                    active: true,
                }
            }else if part_version == "main"{
                Self{
                    name: parts[0].to_string(),
                    version: PackageVersion::Main,
                    active: true,
                }
            }else{
                let version = Version::parse(part_version);
                if version.is_err(){
                    Self{
                        name: parts[0].to_string(),
                        version: PackageVersion::Latest,
                        active: true,
                    }
                }else{
                    Self{
                        name: parts[0].to_string(),
                        version: PackageVersion::Version(version.unwrap()),
                        active: true,
                    }
                }
            }
        }else{
            Self{
                name: package_name.to_string(),
                version: PackageVersion::Latest,
                active: true,
            }
        }
    }

    pub fn from_vec_str(package_name: &Vec<String>) -> Vec<PackageName>{
        package_name.iter().map(|name| Self::from_str(name)).collect()
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
        if self.packages.contains_key(&package.package.name){
            return Err(RepositoryError::PackageAlreadyExists(format!("Package {} has already been linked", package.package.name)));
        }

        self.packages.entry(package.package.name.clone())
            .or_insert_with(|| PackageRegistry::new())
            .link_package(package.package_path.clone());

        self.save()?;
        Ok(())
    }

    pub fn remove_package(&mut self, package_name: &str) -> RepositoryResult<()>{
        self.packages.remove(package_name);
        // remove the folder from the repository
        let package_path = self.repository_path.join(package_name);
        if package_path.exists(){
            std::fs::remove_dir_all(package_path)?;
        }
        self.save()?;
        Ok(())
    }


    pub fn list_packages(&self) -> RepositoryResult<HashMap<String, PackageRegistry>>{
        Ok(self.packages.clone())
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
        if !vat_repository_path.exists(){
            // create the file
            std::fs::File::create(&vat_repository_path)?;
        }

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


    pub fn run(&self, package_name: &PackageName,
        command_name: &str,
        append_env: Option<Vec<PackageName>>,
        detach: bool,
        add_env: Option<HashMap<String, String>>,
        additonal_cmds: Option<Vec<String>>
    ) -> RepositoryResult<()>{
        let package_registry = self.get_package_by_package_name(package_name);
        if package_registry.is_none(){
            return Err(RepositoryError::PackageNotFound(format!("Package {} not found", package_name.name)));
        }
        let package_registry = package_registry.unwrap();
        let package_path = package_registry.get_package_path(package_name);
        if package_path.is_none(){
            return Err(RepositoryError::PackageNotFound(format!("Package {} not found", package_name.name)));
        }
        let package_path = package_path.unwrap();
        let mut vat = Vat::read(package_path)?;
        if append_env.is_some(){
            vat.set_resolved_env(self.resolve_append_env(append_env.unwrap())?);
        }
        dbg!(&vat.resolved_env);
        vat.resolve_env()?;
        dbg!(&vat.resolved_env);
        vat.run(command_name, detach, add_env, additonal_cmds)?;
        Ok(())
    }


    pub fn resolve_stack_env(&self, stack: Stack) ->  HashMap<String, String>{
        let mut package_names : Vec<PackageName> = Vec::new(); 
        // append main package
        package_names.push(stack.package);
        if stack.append.is_some(){
            let append = stack.append.unwrap();
            package_names.extend(append);
        }

        let output_env : HashMap<String, String> = HashMap::new();
        let resolved_output = self.resolve_append_env(package_names);
        if resolved_output.is_err(){
            return output_env;
        }else{
            resolved_output.unwrap()
        }
        

    }


    pub fn resolve_append_env(&self, package_names: Vec<PackageName>) -> RepositoryResult<HashMap<String, String>>{
        let mut resolved_env: HashMap<String, String> = HashMap::new();
        for package_name in package_names{
            let package_registry = self.get_package_by_package_name(&package_name);
            if package_registry.is_none(){
                Console::error(&format!("Error: Package `{}/{}` not found in repository", package_name.name, package_name.version));
            }else{
                Console::info(&format!("Resolving package `{}/{}`", package_name.name, package_name.version));
                let package_registry = package_registry.unwrap();
                let package_path = package_registry.get_package_path(&package_name);
                if package_path.is_some(){
                    let mut vat = Vat::read(package_path.unwrap())?;
                    vat.set_resolved_env(resolved_env.clone());
                    vat.resolve_env()?;
                    resolved_env.extend(vat.resolved_env);
                }
            }

        }
        Ok(resolved_env)
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
        match &package_name.version{
            PackageVersion::Main => Some(self.main_brach_path.clone()),
            PackageVersion::Latest => {
                let version = self.versions.keys().max().unwrap();
                if self.versions.contains_key(&version){
                    return Some(self.versions[&version].package_path.clone());
                }
                None
            }
            PackageVersion::Version(version) => {
                if self.versions.contains_key(&version){
                    return Some(self.versions[&version].package_path.clone());
                }
                None
            }
        }
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