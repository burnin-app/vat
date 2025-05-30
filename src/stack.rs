use crate::repository::{PackageName, Repository};
use crate::errors::{StackError, StackResult};
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


impl Stack{
    pub fn run(self) -> StackResult<()>{
        let repository = Repository::load()?;
        let run_result = repository.run(
                                                        &self.package,
                                                        &self.command,
                                                        self.append,
                                                        true
                                                    );
        match run_result{
            Ok(_) => {
                Ok(())
            }
            Err(e) => {
                Err(StackError::from(e))
            }
        }
    }
}