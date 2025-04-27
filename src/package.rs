use std::path::PathBuf;
use url::Url;
use serde::{Deserialize, Serialize};    
use semver::Version;



#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Package {
    // The name of the package
    pub name: String,
    // The version of the package
    pub version: Version,
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
    pub fn new(name: String) -> Self {
        Self { name, version: Version::new(0, 1, 0), description: None, authors: None, license: None, readme: None, homepage: None, repository: None, documentation: None }
    }
}
