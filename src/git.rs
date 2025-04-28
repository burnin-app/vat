use git2::Repository;
use std::path::PathBuf;
use std::io::Write;
use crate::errors::GitResult;
use crate::errors::GitError;
use semver::Version;
use std::process::Command;

pub struct Git{
    pub repo: Repository,
}

impl Git{
    pub fn init(package_path: PathBuf) -> GitResult<Self>{
        let git_path = package_path.join(".git");
        if git_path.exists(){
            let repo = Repository::open(package_path.clone())?;
            Ok(Self{repo})
        }else{
            let repo = Repository::init(package_path.clone())?;

            // rename master to main
            let mut cmd = Command::new("git");
            cmd.arg("branch");
            cmd.arg("-m");
            cmd.arg("master");
            cmd.arg("main");
            cmd.current_dir(package_path.clone());
            cmd.output()?;

            Ok(Self{repo})
        }
    }

    pub fn get_tags(&self) -> GitResult<Vec<String>>{
        let tags = self.repo.tag_names(None)?;
        let tags = tags.iter().collect::<Vec<_>>();
        let tags = tags.iter().map(|tag| tag.unwrap().to_string()).collect::<Vec<_>>();
        Ok(tags)
    }

    pub fn get_semver_tags(&self) -> GitResult<Vec<Version>>{
        let tags = self.get_tags()?;
        let tags = tags.iter().filter(|tag| tag.parse::<Version>().is_ok()).collect::<Vec<_>>();
        Ok(tags.iter().map(|tag| tag.parse::<Version>().unwrap()).collect::<Vec<_>>())
    }

    pub fn get_latest_semver_tag(&self) -> GitResult<Option<Version>>{
        let tags = self.get_semver_tags()?;
        let tag = tags.iter().max();
        Ok(tag.map(|tag| tag.clone()))
    }

    pub fn git_ignore(&self) -> GitResult<()>{
        let path = self.repo.path().join(".gitignore");
        let mut file = std::fs::File::create(path).map_err(GitError::Io)?;
        let ignore_raw_stirng ="";
        file.write_all(ignore_raw_stirng.as_bytes()).map_err(GitError::Io)?;
        Ok(())
    }
}


