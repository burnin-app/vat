use clap::{Parser, Subcommand};
// use std::process::Command;

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
    Init,
}


fn main() -> Result<(), anyhow::Error> {
    let cli = Cli::parse();

    match cli.command {
        Some(Commands::Init) => {
            println!("Initializing Vat package...");
        }
        None => {
            println!("No command provided");
        }
    }

    Ok(())
}