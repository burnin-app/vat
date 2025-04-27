use std::path::PathBuf;
use std::collections::HashMap;
use url::Url;
use crate::environment::{EnvVar, Environments};
use std::io::Write;
use serde::{Deserialize, Serialize};    


#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Vat{
    pub package: Package,
    pub env: Option<Environments>
}


impl Vat{
    pub fn new(package: Package) -> Self{


        let mut vat = Vat{
            package,
            env: None,
        };


        vat
    }

    pub fn read(package_path: &PathBuf) -> Result<Vat, anyhow::Error> {
        let vat_toml_path = package_path.join("vat.toml");
        if !vat_toml_path.exists() {
            return Err(anyhow::anyhow!("{} given path is not a vat package", package_path.to_str().unwrap_or_default()));
        }
        let toml_string = std::fs::read_to_string(vat_toml_path)?;
        let vat: Vat = toml::from_str(&toml_string)?;
        Ok(vat)
    }


    pub fn save(&self, package_path: &PathBuf) -> Result<Self, anyhow::Error> {
        let toml_string = toml::to_string(self)?;
        let mut toml_file = std::fs::File::create(package_path.join("vat.toml"))?;
        toml_file.write_all(toml_string.as_bytes())?;
        Ok(self.clone())
    }


}


#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Package {
    // The name of the package
    pub name: String,
    // The version of the package
    pub version: String,
    // An optional package description
    pub description: Option<String>,
    // authors of the package
    pub authors: Option<Vec<String>>,
    // license of the package
    pub license: Option<String>,
    // path to the package
    pub readme: Option<PathBuf>,
    // url of the package
    pub homepage: Option<Url>,
    // dependencies of the package
    pub repository: Option<Url>,
    // path to the package
    pub documentation: Option<Url>,

}

impl Package {
    pub fn new(name: String, version: String) -> Self {
        Self { name, version, description: None, authors: None, license: None, readme: None, homepage: None, repository: None, documentation: None }
    }
}
