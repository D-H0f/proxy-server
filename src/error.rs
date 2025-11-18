use thiserror::Error;


#[derive(Debug, Error)]
pub enum ServerError {
    #[error("Failed to parse toml contents")]
    TomlParseError(#[from] toml::de::Error),

    #[error("Failed to read from file")]
    IOError(#[from] std::io::Error),
}
