use thiserror::Error;

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
}

pub type PackageResult<T> = std::result::Result<T, PackageError>;


#[derive(Error, Debug)]
pub enum RepositoryError{
    #[error("Repository not found: {0}")]
    RepositoryNotFound(String),

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
}


pub type GitResult<T> = std::result::Result<T, GitError>;