use thiserror::Error;


#[derive(Error, Debug)]
pub enum StackError{
    #[error("CommandError: {0}")]
    Command(String),

    #[error("Repository Error: {0}")]
    RepositoryError(#[from] RepositoryError)
}

pub type StackResult<T> = std::result::Result<T, StackError>;


#[derive(Error, Debug)]
pub enum PackageError{
    #[error("InitializationError: {0}")]
    InitalizationError(String),

    #[error("Package already exists: {0}")]
    PackageAlreadyExists(String),

    #[error("Error Reading the package: {0}")]
    ReadError(String),

    #[error("No package found in given directory: {0}")]
    PackageNotFound(String),

    #[error("Invalid Package: {0}")]
    InvalidPackage(String),

    #[error("Error writing the package: {0}")]
    WriteError(#[from] std::io::Error),

    #[error("Error parsing the package: {0}")]
    ParseError(#[from] toml::de::Error),

    #[error("Error serializing the package: {0}")]
    SerializeError(#[from] toml::ser::Error),

    #[error("Git Error: {0}")]
    GitError(#[from] GitError),

    #[error("Command not found: {0}")]
    CommandNotFound(String),

    #[error("System time error: {0}")]
    SystemTimeError(#[from] std::time::SystemTimeError),
}

pub type PackageResult<T> = std::result::Result<T, PackageError>;


#[derive(Error, Debug)]
pub enum RepositoryError{
    #[error("Repository not found: {0}")]
    RepositoryNotFound(String),

    #[error("Package not found: {0}")]
    PackageNotFound(String),

    #[error("Error reading the repository: {0}")]
    ReadError(String),

    #[error("Error writing the repository: {0}")]
    WriteError(#[from] std::io::Error),

    #[error("Error parsing the repository: {0}")]
    ParseError(#[from] toml::de::Error),

    #[error("Error serializing the repository: {0}")]
    SerializeError(#[from] toml::ser::Error),

    #[error("Package already exists: {0}")]
    PackageAlreadyExists(String),

    #[error("Error loading the config: {0}")]
    ConfigError(String),

    #[error("Error publishing the package: {0}")]
    PublishError(String),

    #[error("Git Error: {0}")]
    GitError(#[from] GitError),

    #[error("Package Error: {0}")]
    PackageError(#[from] PackageError),

}

pub type RepositoryResult<T> = std::result::Result<T, RepositoryError>;


#[derive(Error, Debug)]
pub enum GitError{
    #[error("Error initializing the repository: {0}")]
    InitError(#[from] git2::Error),

    #[error("Error writing the file: {0}")]
    Io(#[from] std::io::Error),

    #[error("Error executing the git command: {0}")]
    CommandError(String),

    #[error("Error zipping the package: {0}")]
    ZipError(#[from] zip::result::ZipError),

    #[error("Error getting the tag message: {0}")]
    TagMessageError(String),
}


pub type GitResult<T> = std::result::Result<T, GitError>;