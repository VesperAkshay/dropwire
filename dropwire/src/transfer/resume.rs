use crate::error::DropWireError;
use bitvec::prelude::*;
use std::path::{Path, PathBuf};

pub struct PartialState {
    pub file_path: PathBuf,
    pub expected_hash: [u8; 32],
    pub chunk_hashes: Vec<[u8; 32]>,
    pub total_chunks: u64,
    pub received: BitVec<u8, Lsb0>,
}

impl PartialState {
    pub fn new(
        file_path: &Path,
        hash: [u8; 32],
        chunk_hashes: Vec<[u8; 32]>,
        total_chunks: u64,
    ) -> Self {
        let mut received = BitVec::new();
        received.resize(total_chunks as usize, false);
        Self {
            file_path: file_path.to_path_buf(),
            expected_hash: hash,
            chunk_hashes,
            total_chunks,
            received,
        }
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut buf = Vec::new();
        buf.extend_from_slice(&self.expected_hash);
        buf.extend_from_slice(&self.total_chunks.to_be_bytes());
        for h in &self.chunk_hashes {
            buf.extend_from_slice(h);
        }
        let bits = self.received.clone().into_vec();
        buf.extend_from_slice(&(bits.len() as u32).to_be_bytes());
        buf.extend_from_slice(&bits);
        buf
    }

    pub fn from_bytes(file_path: &Path, bytes: &[u8]) -> Result<Self, DropWireError> {
        if bytes.len() < 40 {
            return Err(DropWireError::Protocol("State file too short".into()));
        }
        let mut expected_hash = [0u8; 32];
        expected_hash.copy_from_slice(&bytes[0..32]);
        let mut tc_bytes = [0u8; 8];
        tc_bytes.copy_from_slice(&bytes[32..40]);
        let total_chunks = u64::from_be_bytes(tc_bytes);

        // A sanity check to prevent absurd allocations or OOM (1M chunks = ~1TB of data)
        if total_chunks > 1_000_000 {
            return Err(DropWireError::Protocol("State file corrupted: total_chunks exceeds maximum allowed (1,000,000)".into()));
        }
        
        // Ensure bytes length at least contains the hashes
        let expected_hashes_size = total_chunks.checked_mul(32).ok_or_else(|| {
            DropWireError::Protocol("State file corrupted: integer overflow".into())
        })? as usize;
        
        if bytes.len() < 40 + expected_hashes_size {
            return Err(DropWireError::Protocol("State file truncated at hashes".into()));
        }

        let mut offset = 40;
        let mut chunk_hashes = Vec::with_capacity(total_chunks as usize);
        for _ in 0..total_chunks {
            let mut h = [0u8; 32];
            h.copy_from_slice(&bytes[offset..offset + 32]);
            chunk_hashes.push(h);
            offset += 32;
        }

        if bytes.len() < offset + 4 {
            return Err(DropWireError::Protocol(
                "State file truncated at bitmap len".into(),
            ));
        }
        let mut blen_bytes = [0u8; 4];
        blen_bytes.copy_from_slice(&bytes[offset..offset + 4]);
        let bit_vec_bytes_len = u32::from_be_bytes(blen_bytes) as usize;
        offset += 4;

        if bytes.len() < offset + bit_vec_bytes_len {
            return Err(DropWireError::Protocol(
                "State file truncated at bitmap".into(),
            ));
        }

        let bits_slice = &bytes[offset..offset + bit_vec_bytes_len];
        let mut received = BitVec::<u8, Lsb0>::from_vec(bits_slice.to_vec());
        received.truncate(total_chunks as usize);

        Ok(Self {
            file_path: file_path.to_path_buf(),
            expected_hash,
            chunk_hashes,
            total_chunks,
            received,
        })
    }

    pub fn load(file_path: &Path) -> Result<Option<Self>, DropWireError> {
        if !file_path.exists() {
            return Ok(None);
        }
        let mut file = std::fs::File::open(file_path)?;
        let mut bytes = Vec::new();
        std::io::Read::read_to_end(&mut file, &mut bytes)?;
        let mut state = Self::from_bytes(file_path, &bytes)?;
        state.file_path = file_path.to_path_buf();
        Ok(Some(state))
    }

    pub fn save(&self) -> Result<(), DropWireError> {
        let bytes = self.to_bytes();
        let mut file = std::fs::OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(&self.file_path)?;
        std::io::Write::write_all(&mut file, &bytes)?;
        Ok(())
    }

    pub fn mark_received(&mut self, chunk_index: u64) {
        if (chunk_index as usize) < self.total_chunks as usize {
            self.received.set(chunk_index as usize, true);
        }
    }

    pub fn is_complete(&self) -> bool {
        self.received.all()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    use tempfile::NamedTempFile;

    #[test]
    fn test_save_and_load() {
        let f = NamedTempFile::new().unwrap();
        let path = f.path().to_path_buf();

        let hash = [1u8; 32];
        let chunk_hashes = vec![[2u8; 32], [3u8; 32]];

        let mut state = PartialState::new(&path, hash, chunk_hashes.clone(), 2);
        state.mark_received(1);
        state.save().unwrap();

        let loaded = PartialState::load(&path).unwrap().unwrap();
        assert_eq!(loaded.expected_hash, hash);
        assert_eq!(loaded.chunk_hashes, chunk_hashes);
        assert_eq!(loaded.total_chunks, 2);
        assert!(!loaded.received[0]);
        assert!(loaded.received[1]);
    }

    #[test]
    fn test_mark_chunks() {
        let path = PathBuf::from("dummy");
        let mut state = PartialState::new(&path, [0; 32], vec![[0; 32]; 15], 15);
        state.mark_received(0);
        state.mark_received(5);
        state.mark_received(10);

        assert!(state.received[0]);
        assert!(state.received[5]);
        assert!(state.received[10]);
        assert!(!state.received[1]);
    }

    #[test]
    fn test_is_complete() {
        let path = PathBuf::from("dummy");
        
        let mut state = PartialState::new(&path, [0; 32], vec![[0; 32]; 2], 2);
        assert!(!state.is_complete());
        state.mark_received(0);
        assert!(!state.is_complete());
        state.mark_received(1);
        assert!(state.is_complete());
    }

    #[test]
    fn test_corrupt_state_file() {
        let f = NamedTempFile::new().unwrap();
        let path = f.path().to_path_buf();
        let state_path = path.with_extension("dropwire-partial");
        std::fs::write(&state_path, vec![1, 2, 3]).unwrap();

        let res = PartialState::load(&path);
        assert!(res.is_err());
    }

}
