
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Commands{
    #[serde(flatten)]
    pub global: HashMap<String, String>,
    pub macos: Option<HashMap<String, String>>,
    pub linux: Option<HashMap<String, String>>,
    pub windows: Option<HashMap<String, String>>,
}


#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Command{
    pub value: String
}