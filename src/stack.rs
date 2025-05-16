use crate::repository::PackageName;

#[derive(Debug, Clone)]
pub struct Stack{
    pub command: String,
    pub package: PackageName,
    pub append: Option<Vec<PackageName>>,
    pub detach: bool
}