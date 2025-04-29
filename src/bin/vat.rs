use clap::{Parser, Subcommand};
use std::process::Command;
use vat::Vat;
use vat::console::Console;

const VERSION: &str = env!("CARGO_PKG_VERSION");
const MESSAGE: &str = "Vat is a lightweight package manager / environment manager";

#[derive(Parser)]
#[command(author, version = VERSION, about = MESSAGE, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}


#[derive(Subcommand)]
enum Commands {
    #[command(name = "init", about = "Create a new Vat package in an existing directory")]
    Init,
    #[command(name = "new", about = "Create a new Vat package")]
    New{
        name: String,
    },
    #[command(name = "cat", about = "Read a Vat package")]
    Cat,
    #[command(name = "up", about = "Increment the version of a Vat package, commit and create a new git tag")]
    Up{
            #[arg(short = 'M', long, help = "Increment the major version")]
            major:bool,
            #[arg(short = 'm', long, help = "Increment the minor version")]
            minor:bool,
            #[arg(short = 'p', long, help = "Increment the patch version")]
            patch:bool,
        },
    Test
}


fn main() -> Result<(), anyhow::Error> {
    let cli = Cli::parse();

    match cli.command {
        Some(Commands::Init) => {
            let current_dir = std::env::current_dir()?;
            let output = Vat::init(current_dir, false);
            match output{  
                Ok(vat) => {
                    Console::create_package(&vat.package.name, false);
                }
                Err(e) => {
                    Console::error(&e.to_string());
                }
            }
        }
        Some(Commands::New { name }) => {
            let current_dir = std::env::current_dir()?;
            let path = current_dir.join(&name);
            let output = Vat::init(path, true);
            match output{
                Ok(_) => {
                    Console::create_package(&name, true);
                }
                Err(e) => {
                    Console::error(&e.to_string());
                }
            }
        }
        Some(Commands::Cat) => {
            let current_dir = std::env::current_dir()?;
            let output = Vat::read(current_dir);
            match output{
                Ok(vat) => {
                    println!("{:#?}", vat);
                }
                Err(e) => {
                    Console::error(&e.to_string());
                }
            }
        }
        Some(Commands::Up { major, minor, patch }) => {
            let current_dir = std::env::current_dir()?;
            let output = Vat::read(current_dir);
            match output{
                Ok(mut vat) => {
                    vat.up(major, minor, patch)?;
                    let message = format!("Vat package updated to {}", vat.package.version);
                    Console::success(&message);
                }
                Err(e) => {
                    Console::error(&e.to_string());
                }
            }
        }
        Some(Commands::Test) => {
            let output = Command::new("git")
                .arg("config")
                .arg("--global")
                .arg("user.name")
                .output()
                .expect("failed to execute git command");

            if output.status.success() {
                let value = String::from_utf8_lossy(&output.stdout).trim().to_string();
                println!("{}", value);
                if !value.is_empty() {
                    println!("{}", value);
                }
            }
        }
        None => {
            println!("No command provided");
        }
    }

    Ok(())
}