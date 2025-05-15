use console::style;

pub struct Console;

impl Console{
    pub fn info(message: &str){
        println!("{}", style(message).white());
    }

    pub fn warn(message: &str){
        println!("{}", style(message).yellow());
    }

    pub fn error(message: &str){
        println!("{}", style(message).red());
    }

    pub fn success(message: &str){
        println!("{}", style(message).green());
    }

    pub fn bold(message: &str){
        println!("{}", style(message).bold());
    }

    pub fn dim(message: &str){
        println!("{}", style(message).dim());
    }

    pub fn create_package(name: &str, create: bool){
        let mut key = "Initializing";
        if create{
            key = "Creating";
        }
        let green = style(format!("{}", key)).green();
        println!("{} `{}` vat package", green, name);
        let message = "note: see more `vat.toml` keys and their definitions at https://github.com/burnin-app/vat";
        println!("{}", style(message).dim());
    }

    pub fn error_display(message: &str){
        let key = style(format!("Error:")).red();
        println!("{} {}", key, message);
    }

}

