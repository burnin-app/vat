use crate::repository::PackageName;
use chrono::{DateTime, Utc};

pub struct Stacks{
    pub stacks: Vec<Stack>
}


#[derive(Debug, Clone)]
pub struct Stack{
    pub name: String,
    pub command: String,
    pub package: PackageName,
    pub append: Option<Vec<PackageName>>,
    pub detach: bool,
    pub icon: Option<String>,
    pub modified_at: Option<DateTime<Utc>>,
    pub created_at: Option<DateTime<Utc>>,
}