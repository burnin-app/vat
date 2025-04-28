use std::path::PathBuf;
use serde::{Deserialize, Serialize};
use std::io::Write;
use fs2::FileExt;
use std::fs::OpenOptions;

use crate::command::Commands;
use crate::package::Package;
use crate::environment::Environments;
use crate::dependencies::Dependency;
use crate::errors::{PackageResult, PackageError};
use crate::git::Git;

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

    pub fn get_package_path(&self) -> PathBuf{
        self.package_path.clone()
    }


}
