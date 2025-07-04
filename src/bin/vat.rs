use clap::{Parser, Subcommand};
use vat::Vat;
use vat::repository::{Repository, PackageName};
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
    #[command(name = "publish", about = "Publish package to the repository")]
    Publish{
        #[arg(short = 'm', long, help = "The message to publish the package with")]
        message: String,
        // #[arg(short, long)]
        // remote: bool,
    },
    // link
    #[command(name = "link", about = "Link current package to the repository")]
    Link,
    #[command(name = "run", about = "Run a Vat package")]
    Run{
        name: String,
        #[arg(long="package", short='p', help = "The package to run the command in")]
        package: Option<String>,
        #[arg(long="append", short='a', num_args = 1.., help = "Append packages to the environment")]
        append: Option<Vec<String>>,
        #[arg(short, long, default_value = "false")]
        detach: bool,
    },
    #[command(name = "list", about = "List all packages in the repository")]
    List,
    #[command(name = "remove", about = "Remove a package from the repository")]
    Remove{
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
            let repository = Repository::load()?;
            dbg!(&repository);
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
        Some(Commands::Link)=>{
            let current_dir = std::env::current_dir()?;
            let package_read = Vat::read(current_dir);
            match package_read{
                Ok(package)  => {
                    let mut repository = Repository::load()?;
                    let link_result = repository.link_package(package);
                    match link_result{
                        Ok(_) => {
                            Console::success(&format!("Package linked successfully to the repository"));
                        }
                        Err(e) => {
                            Console::error(&e.to_string());
                        }
                    }
                }
                Err(e)=> {
                    Console::error(&e.to_string());
                }
            }
        }
        Some(Commands::List)=>{
            let repository = Repository::load()?;
            println!("Repository: {}", repository.repository_path.display());
            let packages = repository.list_packages()?;
            for (package_name, package_registry) in packages{
                println!("{}", package_name);
                for (version, _) in package_registry.versions{
                    println!("  {}", version.to_string());
                }
            }
        }
        Some(Commands::Remove{name})=>{
            let mut repository = Repository::load()?;
            // ask for confirmation 
            Console::info(&format!("Are you sure you want to remove {} from the repository? (y/n)", name));
            let mut input = String::new();
            std::io::stdin().read_line(&mut input)?;
            if input.trim().to_lowercase() != "y" {
                Console::info("Operation cancelled");
                return Ok(());
            }
            let remove_result = repository.remove_package(&name);
            match remove_result{
                Ok(_)=>{
                    Console::success(&format!("{} removed successfully from the repository", name));
                }
                Err(e)=>{
                    Console::error(&e.to_string());
                }
            }
        }
        Some(Commands::Run { name, package, append, detach }) => {
            let current_dir = std::env::current_dir()?;
            let append = if append.is_some(){
                Some(PackageName::from_vec_str(&append.unwrap()))
            }else{
                None
            };

            if package.is_none(){
                let output = Vat::read(current_dir);
                match output{
                    Ok(mut vat) => {
                        if append.is_some(){
                            let repository = Repository::load()?;
                            let resolved_env = repository.resolve_append_env(append.unwrap())?;
                            vat.set_resolved_env(resolved_env);
                        }
                        vat.resolve_env()?;
                        vat.run(&name, detach, None)?;
                    }
                    Err(e) => {
                        Console::error(&e.to_string());
                    }
                }
            }else{
                let repository = Repository::load()?;
                let package_name = package.unwrap();
                let package_name = PackageName::from_str(&package_name);
                let run_result = repository.run(&package_name, &name, append, detach, None);
                match run_result{
                    Ok(_) => {
                    }
                    Err(e) => {
                        Console::error(&e.to_string());
                    }
                }
            }
        }
        None => {
            println!("No command provided");
        }
    }

    Ok(())
}