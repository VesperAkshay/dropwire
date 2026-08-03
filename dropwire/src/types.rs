use crate::error::DropWireError;
use std::collections::HashSet;
use std::sync::OnceLock;
use zeroize::{Zeroize, ZeroizeOnDrop};

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct CodePhrase(pub String);
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct ChannelId(pub String);

impl ChannelId {
    pub fn derive(code: &str) -> Self {
        if let Some(idx) = code.find('-') {
            Self(code[..idx].to_string())
        } else {
            Self(code.to_string())
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ChunkIdx(pub u64);

#[derive(Zeroize, ZeroizeOnDrop)]
pub struct AuthToken(pub [u8; 16]);

#[derive(Zeroize, ZeroizeOnDrop)]
pub struct StreamKey(pub [u8; 32]);

#[derive(Zeroize, ZeroizeOnDrop)]
pub struct SharedSecret(pub [u8; 32]);

use serde::{Deserialize, Serialize};

pub const CHUNK_SIZE: usize = 1_048_576;
pub const MAX_STREAMS: u8 = 8;
pub const DEFAULT_STREAMS: u8 = 4;
pub const FRAME_MAX_SIZE: u32 = 16_777_216;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct FileMetadata {
    pub relative_path: String,
    pub size: u64,
    pub chunks: u64,
}

pub fn safe_path(untrusted: &str) -> Result<std::path::PathBuf, DropWireError> {
    use std::path::{Component, Path, PathBuf};
    let p = Path::new(untrusted);
    if p.is_absolute() {
        return Err(DropWireError::Protocol("Absolute path rejected".into()));
    }
    let mut safe = PathBuf::new();
    for comp in p.components() {
        match comp {
            Component::Normal(c) => {
                let s = c.to_string_lossy();
                if s.contains(':') || s.contains("..") {
                    return Err(DropWireError::Protocol("Invalid path component".into()));
                }
                safe.push(c);
            }
            Component::CurDir => {},
            _ => return Err(DropWireError::Protocol("Path traversal detected".into())),
        }
    }
    Ok(safe)
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TransferManifest {
    pub files: Vec<FileMetadata>,
    pub total_size: u64,
    pub total_chunks: u64,
    pub overall_hash: [u8; 32],
}

static WORDLIST: OnceLock<HashSet<&'static str>> = OnceLock::new();

fn get_wordlist() -> &'static HashSet<&'static str> {
    WORDLIST.get_or_init(|| {
        crate::wordlist::BIP39_WORDS
            .iter()
            .copied()
            .collect()
    })
}

impl CodePhrase {
    pub fn new(phrase: &str) -> Result<Self, DropWireError> {
        let parts: Vec<&str> = phrase.split('-').collect();
        if parts.len() != 3 {
            return Err(DropWireError::InvalidCodePhrase(phrase.to_string()));
        }

        let num_str = parts[0];
        if num_str.starts_with('0') {
            return Err(DropWireError::InvalidCodePhrase(phrase.to_string()));
        }

        let num: u32 = num_str
            .parse()
            .map_err(|_| DropWireError::InvalidCodePhrase(phrase.to_string()))?;
        if num == 0 || num > 999 {
            return Err(DropWireError::InvalidCodePhrase(phrase.to_string()));
        }

        let wordlist = get_wordlist();
        for &word in &parts[1..] {
            if word != word.to_lowercase() {
                return Err(DropWireError::InvalidCodePhrase(phrase.to_string()));
            }
            if !wordlist.contains(word) {
                return Err(DropWireError::InvalidCodePhrase(phrase.to_string()));
            }
        }

        Ok(Self(phrase.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_code_phrase() {
        let result = CodePhrase::new("7-guitar-abandon");
        assert!(result.is_ok());
    }

    #[test]
    fn test_invalid_format() {
        let result = CodePhrase::new("invalid");
        assert!(matches!(result, Err(DropWireError::InvalidCodePhrase(_))));
    }

    #[test]
    fn test_number_too_large() {
        let result = CodePhrase::new("1000-guitar-abandon");
        assert!(matches!(result, Err(DropWireError::InvalidCodePhrase(_))));
    }

    #[test]
    fn test_number_leading_zero() {
        let result = CodePhrase::new("07-guitar-abandon");
        assert!(matches!(result, Err(DropWireError::InvalidCodePhrase(_))));
    }

    #[test]
    fn test_number_zero() {
        let result = CodePhrase::new("0-guitar-abandon");
        assert!(matches!(result, Err(DropWireError::InvalidCodePhrase(_))));
    }

    #[test]
    fn test_invalid_word() {
        let result = CodePhrase::new("1-INVALIDWORD-word");
        assert!(matches!(result, Err(DropWireError::InvalidCodePhrase(_))));
    }
}
