use serde::{Serialize, ser::Serializer};

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Io(#[from] std::io::Error),

    #[error("Database error: {0}")]
    Database(String),

    #[error("Network request error: {0}")]
    Reqwest(#[from] reqwest::Error),

    #[error("HTML parsing error: {0}")]
    Scraper(String),

    #[error("Tauri SQL plugin error: {0}")]
    TauriSql(String),
}

impl Serialize for Error {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(self.to_string().as_ref())
    }
}

// Define a specific result type for our application
pub type Result<T> = std::result::Result<T, Error>;
