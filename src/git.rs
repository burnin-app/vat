use git2::Repository;
use std::path::PathBuf;
use crate::errors::GitResult;


pub struct Git{
    pub repo: Repository,
}

impl Git{
    pub fn init(path: PathBuf) -> GitResult<Self>{
        let git_path = path.join(".git");
        if git_path.exists(){
            let repo = Repository::open(path)?;
            Ok(Self{repo})
        }else{
            let repo = Repository::init(path)?;
            Ok(Self{repo})
        }
    }
}


