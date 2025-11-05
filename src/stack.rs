use crate::repository::{self, PackageName, Repository};
use crate::errors::{StackError, StackResult};
use chrono::{DateTime, Utc};
use std::collections::HashMap;

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
    pub fn run(self, add_env: Option<HashMap<String, String>>, additonal_cmds: Option<Vec<String>>) -> StackResult<()>{
        let repository = Repository::load()?;
        let run_result = repository.run(
                                                        &self.package,
                                                        &self.command,
                                                        self.append,
                                                        true,
                                                        add_env,
                                                        additonal_cmds
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

    pub fn env_from_stack(self) -> StackResult<HashMap<String, String>>{
        let repository = Repository::load()?;
        let result_env = repository.resolve_stack_env(self);
        Ok(result_env)
    }
}