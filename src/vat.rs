use std::path::PathBuf;
use serde::{Deserialize, Serialize};
use std::io::Write;
use fs2::FileExt;
use std::fs::OpenOptions;
use semver::Version;
use std::collections::HashMap;
use std::process::Stdio;
use dirs;

use crate::command::Commands;
use crate::package::Package;
use crate::environment::{Environments, EnvVar};
use crate::dependencies::Dependency;
use crate::errors::{PackageResult, PackageError};
use crate::git::Git;
use crate::console::Console;
use crate::variables::Variables;

const VAT_FILE: &str = "vat.toml";  

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Vat{
    #[serde(skip)]
    pub package_path: PathBuf,
    pub package: Package,
    pub variables: Option<Variables>,
    pub env: Option<Environments>,
    pub cmd: Option<Commands>,
    pub dependencies: Option<Vec<Dependency>>,
    #[serde(skip)]  
    pub resolved_env: HashMap<String, String>,
}


impl Vat{
    pub fn new(package: Package) -> Self{
        let vat = Vat{
            package_path: PathBuf::new(),
            package,
            variables: None,
            env: None,
            cmd: None,
            dependencies: None,
            resolved_env: HashMap::new(),
        };
        vat
    }

    pub fn init(path: PathBuf, create: bool) -> PackageResult<Vat>{

        let package_name = path.file_name();
        if package_name.is_none(){
            return Err(PackageError::InvalidPackage("Path is not a valid package".to_string()));
        }

        let package_name = package_name.unwrap().to_string_lossy().to_string();
        let package = Package::new(package_name);

        let mut vat = Vat::new(package);
        vat.package_path = path.clone();
        if vat.is_package(){
            let path = path.join(VAT_FILE);
            return Err(PackageError::PackageAlreadyExists(path.to_string_lossy().to_string()));
        }else{

            if !create{
                if !path.exists(){
                    return Err(PackageError::PackageNotFound(path.to_string_lossy().to_string()));
                }
            }else{
                std::fs::create_dir_all(&path)?;
            }

            let _ = Git::init(path.clone())?;
            vat.save()?;
            return Ok(vat);
        }

    }


    pub fn up_prompt(&mut self, major: bool, minor: bool, patch: bool) -> PackageResult<()>{
        let git = Git::init(self.package_path.clone());
        if git.is_err(){
            let error = git.err().unwrap();
            Console::error(&error.to_string());
            return Err(PackageError::GitError(error));
        }
        let git = git.unwrap();
        let latest_tag = git.get_latest_semver_tag()?;

        // set the latest tag as the current version
        if latest_tag != None{
            self.package.version = latest_tag.unwrap();
            self.increment_version(major, minor, patch);
        }else{
            self.package.version = semver::Version::new(0, 0, 0);
            self.increment_version(major, minor, patch);
        }

        Ok(())
    }


    pub fn up(&mut self, commit_message: &str) -> PackageResult<()>{
        let git = Git::init(self.package_path.clone());

        if git.is_err(){
            let error = git.err().unwrap();
            Console::error(&error.to_string());
            return Err(PackageError::GitError(error));
        }
        let git = git.unwrap();

        // commit vat.toml file
        self.save()?;
        git.add_toml()?;
        git.commit(commit_message)?;
        git.tag(&self.package.version.to_string(), commit_message)?;

        Ok(())
    }



    pub fn is_package(&self) -> bool{
        self.package_path.join(VAT_FILE).exists()
    }

    pub fn read(path: PathBuf) -> PackageResult<Vat> {
        let vat_toml_path = path.join(VAT_FILE);
        if !vat_toml_path.exists() {
            return Err(PackageError::PackageNotFound("Cannot find vat.toml".to_string()));
        }
        
        let file = OpenOptions::new()
            .read(true)
            .open(&vat_toml_path)?;
        
        FileExt::lock_shared(&file)?;
        
        let toml_string = std::fs::read_to_string(vat_toml_path)?;
        let mut vat: Vat = toml::from_str(&toml_string)?;
        vat.package_path = path;
        
        FileExt::unlock(&file)?;
        
        Ok(vat)
    }


    pub fn save(&self) -> PackageResult<Self> {
        let vat_toml_path = self.package_path.join(VAT_FILE);
        let mut file = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(&vat_toml_path)?;
        
        FileExt::lock_exclusive(&file)?;
        
        let toml_string = toml::to_string(self)?;
        file.write_all(toml_string.as_bytes())?;
        
        FileExt::unlock(&file)?;
        
        Ok(self.clone())
    }

    pub fn package_path(&self) -> PathBuf{
        self.package_path.clone()
    }

    pub fn package_version(&self) -> Version{
        self.package.version.clone()
    }


    pub fn increment_version(&mut self, major: bool, minor: bool, patch: bool) {
        let version_parts = self.package.version.clone();
        let major_version = version_parts.major;
        let minor_version = version_parts.minor;
        let patch_version = version_parts.patch;

        if major {
            // Increment major version and reset minor and patch
            self.package.version = semver::Version::new(major_version + 1, 0, patch_version);
        } else if minor {
            // Increment minor version and reset patch
            self.package.version = semver::Version::new(major_version, minor_version + 1, patch_version);
        } else if patch{
            // Increment patch version
            self.package.version = semver::Version::new(major_version, minor_version, patch_version + 1);
        } 
    }

    pub fn set_resolved_env(&mut self, resolved_env: HashMap<String, String>){
        self.resolved_env = resolved_env;
    }


    pub fn resolve_env(&mut self) -> PackageResult<()>{
        // current os
        let current_os = std::env::consts::OS;
        let mut resolved_env = self.resolved_env.clone();

        let dilimeter = if current_os == "windows"{
            ";"
        }else{
            ":"
        };

        Console::resolved_env_list(&self.package.name, "Resolving Environment Variables");

        if let Some(env) = &self.env{
            // process global env
            for (key, env_var) in &env.global{
                self.process_env(key, env_var,&mut resolved_env, &dilimeter);
            }

            // process macos
            if current_os == "macos"{
                if let Some(macos_env) = &env.macos{
                    for (key, env_var) in macos_env{
                        self.process_env(key, env_var,&mut resolved_env, &dilimeter);
                    }
                }
            }else if current_os == "windowsi"{
                if let Some(windows_env) = &env.windows{
                    for (key, env_var) in windows_env{
                        self.process_env(key, env_var,&mut resolved_env, &dilimeter);
                    }
                }
            }else if current_os == "linuxk"{
                if let Some(linux_env) = &env.linux{
                    for (key, env_var) in linux_env{
                        self.process_env(key, env_var,&mut resolved_env, &dilimeter);
                    }
                }
            }
        }

        // remove first dilimeter and add & at the end
        for (key, value) in &mut resolved_env{
            if value.starts_with(dilimeter){
                *value = value.replace(dilimeter, "");
            }
            *value = format!("{}{}&", value, dilimeter);
        }

        self.resolved_env = resolved_env;
        Ok(())
    }


    pub fn run(&self, command_name: &str, detach: bool) -> PackageResult<()>{
        if let Some(cmd) = &self.cmd{
            let command = cmd.get_command(command_name);
            if let Some(command) = command{
                let length = command.len();
                if length > 0 {
                    let first_commnad = command.first();
                    if let Some(first_commnad) = first_commnad{
                        let mut command_process = std::process::Command::new(first_commnad);
                        for arg in &command[1..] {
                            command_process.arg(arg);
                        }
                        let mut resolved_env = self.resolved_env.clone();
                        #[cfg(target_os = "macos")]
                        {
                            if let Some(path) = resolved_env.get_mut("PATH") {
                                *path = expand_tilde_in_path(path);
                            }
                        }
                        command_process.envs(&resolved_env);
                        Console::dim(&format!("Package Name: {}", self.package.name));
                        Console::dim(&format!("Package Path: {}", self.package_path.to_string_lossy().to_string()));
                        Console::info(&format!("Running command: {:?} from package: {}", command_name, self.package.name));
                        if detach{
                            command_process
                            .stdout(Stdio::null())
                            .stderr(Stdio::null())
                            .stdin(Stdio::null());
                            command_process.spawn()?;
                        }else{
                            command_process.output()?;
                        }
                        Ok(())
                    }else{
                        Err(PackageError::CommandNotFound(command_name.to_string()))
                    }
                }else{
                    Err(PackageError::CommandNotFound(command_name.to_string()))
                }
            }else{
                Err(PackageError::CommandNotFound(command_name.to_string()))
            }
        }else{
            Err(PackageError::CommandNotFound(command_name.to_string()))
        }
    }



    pub fn process_env(&self,key:&String, env_var: &EnvVar, resolved_env: &mut HashMap<String, String>, dilimeter: &str){
        for value in &env_var.values{
            let existing_env_values = if resolved_env.contains_key(key){
                resolved_env.get(key).unwrap().clone()
            }else{
                std::env::var(key).unwrap_or_default()
            };

            dbg!(&existing_env_values);

            let value = self.path_resolve(value);
            Console::resolved_env(key, &value);

            if value.starts_with("append:"){
                let value = value.replace("append:", "");
                resolved_env.insert(key.clone(), format!("{}{}{}", existing_env_values, dilimeter, value));
            }else if value.starts_with("prepend:"){
                let value = value.replace("prepend:", "");
                resolved_env.insert(key.clone(), format!("{}{}{}", value, dilimeter, existing_env_values));
            }else if value.starts_with("set:"){
                let value = value.replace("set:", "");
                resolved_env.insert(key.clone(), value);
            }else{
                resolved_env.insert(key.clone(), format!("{}{}{}", existing_env_values, dilimeter, value));
            }
        }
    }

    pub fn path_resolve(&self, path: &str) -> String{
        if path.contains("{root}"){
            let root = self.package_path.clone();
            path.replace("{root}", &root.to_string_lossy().to_string())
        }else{
            path.to_string()
        }
    }


}

fn expand_tilde_in_path(path: &str) -> String {
    let home = dirs::home_dir().map(|p| p.to_string_lossy().to_string()).unwrap_or_default();
    path.split(':')
        .map(|p| {
            if p.starts_with("~") {
                p.replacen("~", &home, 1)
            } else {
                p.to_string()
            }
        })
        .collect::<Vec<_>>()
        .join(":")
}
