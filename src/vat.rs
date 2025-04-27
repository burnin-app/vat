use std::path::PathBuf;
use serde::{Deserialize, Serialize};
use std::io::Write;
use fs2::FileExt;
use std::fs::OpenOptions;

use crate::command::Commands;
use crate::package::Package;
use crate::environment::Environments;
use crate::dependencies::Dependency;

const VAT_FILE: &str = "vat.toml";  

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Vat{
    pub package: Package,
    pub env: Option<Environments>,
    pub cmd: Option<Commands>,
    pub dependencies: Option<Vec<Dependency>>,
}


impl Vat{
    pub fn new(package: Package) -> Self{

        let vat = Vat{
            package,
            env: None,
            cmd: None,
            dependencies: None,
        };

        vat
    }

    pub fn read(package_path: &PathBuf) -> Result<Vat, anyhow::Error> {
        let vat_toml_path = package_path.join(VAT_FILE);
        if !vat_toml_path.exists() {
            return Err(anyhow::anyhow!("{} given path is not a vat package", package_path.to_str().unwrap_or_default()));
        }
        
        let file = OpenOptions::new()
            .read(true)
            .open(&vat_toml_path)?;
        
        FileExt::lock_shared(&file)?;
        
        let toml_string = std::fs::read_to_string(vat_toml_path)?;
        let vat: Vat = toml::from_str(&toml_string)?;
        
        FileExt::unlock(&file)?;
        
        Ok(vat)
    }


    pub fn save(&self, package_path: &PathBuf) -> Result<Self, anyhow::Error> {
        let vat_toml_path = package_path.join(VAT_FILE);
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


}
