use serde::{Deserialize, Serialize};    
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Environments{
    pub macos: Option<HashMap<String, EnvVar>>,
    pub linux: Option<HashMap<String, EnvVar>>,
    pub windows: Option<HashMap<String, EnvVar>>,
    #[serde(flatten)]
    pub global: HashMap<String, EnvVar>,
} 


#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EnvVar{
    pub values: Vec<String>,
}


#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Action{
    Append, 
    Prepend,
    Set
}