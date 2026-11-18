use crate::models::Vocabulary;
use std::fs;
use std::path::Path;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum VocabularyError {
    #[error("Failed to read vocabulary file: {0}")]
    Io(#[from] std::io::Error),
    #[error("Failed to parse vocabulary JSON: {0}")]
    Json(#[from] serde_json::Error),
}

pub struct VocabularyLoader;

impl VocabularyLoader {
    /// Loads vocabulary from a given file path
    pub fn load_from_file<P: AsRef<Path>>(path: P) -> Result<Vocabulary, VocabularyError> {
        let content = fs::read_to_string(path)?;
        let vocab: Vocabulary = serde_json::from_str(&content)?;
        Ok(vocab)
    }

    /// Loads the embedded default vocabulary
    pub fn load_embedded() -> Result<Vocabulary, VocabularyError> {
        let content = include_str!("../vocabulary.json");
        let vocab: Vocabulary = serde_json::from_str(content)?;
        Ok(vocab)
    }
}
