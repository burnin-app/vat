use clap::{Parser, Subcommand};
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
        None => {
            println!("No command provided again");
        }
    }

    Ok(())
}