use clap::{Parser, Subcommand};
use std::process::Command;
use vat::Vat;
use vat::repository::Repository;
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
    #[command(name = "publish", about = "Publish a Vat package to the repository")]
    Publish{
        #[arg(short = 'm', long, help = "The message to publish the package with")]
        message: String,
        // #[arg(short, long)]
        // remote: bool,
    },
    #[command(name = "run", about = "Run a Vat package")]
    Run{
        name: String,
    },
    Test,
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
        Some(Commands::Up { major, mut minor, patch }) => {
            let current_dir = std::env::current_dir()?;
            let output = Vat::read(current_dir);

            if major == false && minor == false && patch == false {
                minor = true;
            }

            match output{
                Ok(mut vat) => {
                    vat.up_prompt(major, minor, patch)?;
                    let new_version = vat.package.version.clone();

                    Console::info(&format!("Do you want to update the vat package to the version {}? (y/n)", new_version));
                    let mut input = String::new();
                    std::io::stdin().read_line(&mut input)?;
                    if input.trim().to_lowercase() == "y" {

                        // get the commit message
                        Console::info("Enter the commit message");
                        let mut commit_message = String::new();
                        std::io::stdin().read_line(&mut commit_message)?;

                        vat.up(&commit_message)?;
                        let message = format!("Vat package updated to {}", vat.package.version);
                        Console::success(&message);
                    }
                }
                Err(e) => {
                    Console::error(&e.to_string());
                }
            }
        }
        Some(Commands::Test) => {
            let current_dir = std::env::current_dir()?;
            let output = Vat::read(current_dir);
            match output{
                Ok(mut vat) => {
                    vat.resolve_env()?;
                }
                Err(e) => {
                    Console::error(&e.to_string());
                }
            }
        }
        Some(Commands::Publish { message }) => {
            let current_dir = std::env::current_dir()?;
            let package_read = Vat::read(current_dir);
            match package_read{
                Ok(package) => {
                    let mut repository = Repository::load()?;
                    let publish_result = repository.publish(package, &message);
                    match publish_result{
                        Ok(_) => {
                            Console::success(&format!("Package published successfully to the repository"));
                        }
                        Err(e) => {
                            Console::error(&e.to_string());
                        }
                    }
                }
                Err(e) => {
                    Console::error(&e.to_string());
                }
            }
        }
        Some(Commands::Run { name }) => {
            let current_dir = std::env::current_dir()?;
            let output = Vat::read(current_dir);
            match output{
                Ok(mut vat) => {
                    vat.resolve_env()?;
                    dbg!(&vat.resolved_env);
                    vat.run(&name)?;
                }
                Err(e) => {
                    Console::error(&e.to_string());
                }
            }
        }
        None => {
            println!("No command provided");
        }
    }

    Ok(())
}