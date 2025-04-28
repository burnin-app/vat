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
pub enum GitError{
    #[error("Error initializing the repository: {0}")]
    InitError(#[from] git2::Error),
}


pub type GitResult<T> = std::result::Result<T, GitError>;