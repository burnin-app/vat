use crate::errors::GitResult;
use crate::errors::GitError;
use crate::console::Console;

use std::fs;
use semver::Version;
use std::process::Command;
use git2::Repository;
use std::path::PathBuf;
use std::io::Write;

pub struct Git{
    pub repo: Repository,
    pub path: PathBuf,
}

impl Git{
    pub fn init(package_path: PathBuf) -> GitResult<Self>{
        let git_path = package_path.join(".git");
        if git_path.exists(){
            let repo = Repository::open(package_path.clone())?;
            Ok(Self{repo, path: package_path})
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

            Ok(Self{repo, path: package_path})
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


    pub fn add_toml(&self) -> GitResult<()>{
        let status = Command::new("git")
        .arg("add")
        .arg("vat.toml")
        .current_dir(&self.path)
        .status()
        .expect("Failed to execute git add");

        if !status.success(){
            let message = format!("Failed to execute git add: {}", status);
            return Err(GitError::CommandError(message));
        }

        Ok(())
    }

    pub fn commit(&self) -> GitResult<()>{
        let status = Command::new("git")
            .arg("commit")
            .arg("-m")
            .arg("Increment version")
            .current_dir(&self.path)
            .status()
            .expect("Failed to execute git commit");

        if !status.success(){
            let message = format!("Failed to execute git commit: {}", status);
            return Err(GitError::CommandError(message));
        }

        Ok(())
    }

    pub fn tag(&self, tag: &str) -> GitResult<()>{
        let status = Command::new("git")
            .arg("tag")
            .arg(tag)
            .current_dir(&self.path)
            .status()
            .expect("Failed to execute git tag");

        if !status.success(){
            let message = format!("Failed to execute git tag: {}", status);
            return Err(GitError::CommandError(message));
        }

        Ok(())
    }

    pub fn zip_tag(&self, package_version: Version, package_path: PathBuf, repository_path: PathBuf) -> GitResult<()>{

        let zip_file_name = format!("{}-package.zip", &package_version);

        Console::info(&format!("Zipping package version: {}", &package_version));
        Console::dim(&format!("This might take a while..."));

        // create zip from git version
        // "git archive --format=zip -o archive.zip 0.0.3"
        let _command = std::process::Command::new("git")
            .arg("archive")
            .arg("--format=zip")
            .arg("-o")
            .arg(&zip_file_name)
            .arg(package_version.to_string())
            .current_dir(&package_path)
            .status()
            .expect("Failed to create zip file");

        // copy zip file to repository
        let mut copy_options = fs_extra::dir::CopyOptions::new();
        copy_options.overwrite = true;
        copy_options.copy_inside = true;

        if !repository_path.exists(){
            fs::create_dir_all(&repository_path)?;
        }

        let source_zip_file_path = package_path.join(zip_file_name);
        let repo_package_version_path = repository_path.join(package_version.to_string());

        let file = std::fs::File::open(&source_zip_file_path)?;
        let mut archive = zip::read::ZipArchive::new(file)?;
        archive.extract(repo_package_version_path)?;

        Ok(())
    }
}


