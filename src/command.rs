
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


impl Commands{
    pub fn get_command(&self, command: &str) -> Option<String>{
        // get the global command
        let mut output = None;
        let global_command = self.global.get(command);
        if let Some(command) = global_command{
            output = Some(command.clone());
        }

        // check for os
        let current_os = std::env::consts::OS;
        if current_os == "windows"{
            if let Some(windows_command) = self.windows.as_ref(){
                let cmd  = windows_command.get(command);
                if let Some(cmd) = cmd{
                    output = Some(cmd.clone());
                }
            }
        }else if current_os == "macos"{
            if let Some(macos_command) = self.macos.as_ref(){
                let cmd  = macos_command.get(command);
                if let Some(cmd) = cmd{
                    output = Some(cmd.clone());
                }
            }
        }else if current_os == "linux"{
            if let Some(linux_command) = self.linux.as_ref(){
                let cmd  = linux_command.get(command);
                if let Some(cmd) = cmd{
                    output = Some(cmd.clone());
                }
            }
        }
        output
    }
}


#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Command{
    pub value: String
}

impl Command{
    pub fn new(value: String) -> Self{
        Self{value}
    }

}