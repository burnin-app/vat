use std::path::PathBuf;
use serde::{Deserialize, Serialize};
use std::io::Write;
use fs2::FileExt;
use std::fs::OpenOptions;
use semver::Version;

use crate::command::Commands;
use crate::package::Package;
use crate::environment::Environments;
use crate::dependencies::Dependency;
use crate::errors::{PackageResult, PackageError};
use crate::git::Git;
use crate::console::Console;

const VAT_FILE: &str = "vat.toml";  

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Vat{
    #[serde(skip)]
    pub package_path: PathBuf,
    pub package: Package,
    pub env: Option<Environments>,
    pub cmd: Option<Commands>,
    pub dependencies: Option<Vec<Dependency>>,
}


impl Vat{
    pub fn new(package: Package) -> Self{
        let vat = Vat{
            package_path: PathBuf::new(),
            package,
            env: None,
            cmd: None,
            dependencies: None,
        };
        vat
    }

    pub fn init(path: PathBuf, create: bool) -> PackageResult<Vat>{

        let package_name = path.file_name();
        if package_name.is_none(){
            return Err(PackageError::InvalidPackage("Path is not a valid package".to_string()));
        }

        let package_name = package_name.unwrap().to_string_lossy().to_string();
        let package = Package::new(package_name);

        let mut vat = Vat::new(package);
        vat.package_path = path.clone();
        if vat.is_package(){
            let path = path.join(VAT_FILE);
            return Err(PackageError::PackageAlreadyExists(path.to_string_lossy().to_string()));
        }else{

            if !create{
                if !path.exists(){
                    return Err(PackageError::PackageNotFound(path.to_string_lossy().to_string()));
                }
            }else{
                std::fs::create_dir_all(&path)?;
            }

            let _ = Git::init(path.clone())?;
            vat.save()?;
            return Ok(vat);
        }

    }


    pub fn up(&mut self, major: bool, minor: bool, patch: bool) -> PackageResult<Self>{
        let git = Git::init(self.package_path.clone());

        if git.is_err(){
            let error = git.err().unwrap();
            Console::error(&error.to_string());
            return Err(PackageError::GitError(error));
        }
        let git = git.unwrap();
        let latest_tag = git.get_latest_semver_tag()?;

        let mut vat = self.clone();
        if latest_tag != None{
            vat.increment_version(major, minor, patch);
            vat.save()?;
        }

        // commit vat.toml file
        git.add_toml()?;
        git.commit()?;
        git.tag(&vat.package.version.to_string())?;

        return Ok(vat);

    }



    pub fn is_package(&self) -> bool{
        self.package_path.join(VAT_FILE).exists()
    }

    pub fn read(path: PathBuf) -> PackageResult<Vat> {
        let vat_toml_path = path.join(VAT_FILE);
        if !vat_toml_path.exists() {
            return Err(PackageError::PackageNotFound("Cannot find vat.toml".to_string()));
        }
        
        let file = OpenOptions::new()
            .read(true)
            .open(&vat_toml_path)?;
        
        FileExt::lock_shared(&file)?;
        
        let toml_string = std::fs::read_to_string(vat_toml_path)?;
        let mut vat: Vat = toml::from_str(&toml_string)?;
        vat.package_path = path;
        
        FileExt::unlock(&file)?;
        
        Ok(vat)
    }


    pub fn save(&self) -> PackageResult<Self> {
        let vat_toml_path = self.package_path.join(VAT_FILE);
        let mut file = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(&vat_toml_path)?;
        
        FileExt::lock_exclusive(&file)?;
        
        let toml_string = toml::to_string(self)?;
        file.write_all(toml_string.as_bytes())?;
        
        FileExt::unlock(&file)?;
        
        Ok(self.clone())
    }

    pub fn package_path(&self) -> PathBuf{
        self.package_path.clone()
    }

    pub fn package_version(&self) -> Version{
        self.package.version.clone()
    }


    pub fn increment_version(&mut self, major: bool, minor: bool, patch: bool) {
        let version_parts = self.package.version.clone();
        let major_version = version_parts.major;
        let minor_version = version_parts.minor;
        let patch_version = version_parts.patch;

        if major {
            // Increment major version and reset minor and patch
            self.package.version = semver::Version::new(major_version + 1, 0, 0);
        } else if minor {
            // Increment minor version and reset patch
            self.package.version = semver::Version::new(major_version, minor_version + 1, 0);
        } else if patch{
            // Increment patch version
            self.package.version = semver::Version::new(major_version, minor_version, patch_version + 1);
        } 
    }


}
