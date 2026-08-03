use crate::types::{FileMetadata, CHUNK_SIZE};
use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

pub struct FileEntry {
    pub path: PathBuf,
    pub relative_path: String,
    pub size: u64,
    pub chunks: u64,
}

pub struct Chunker {
    pub files: Vec<FileEntry>,
    pub total_size: u64,
    pub total_chunks: u64,
    pub chunk_hashes: Vec<[u8; 32]>,
    pub overall_hash: [u8; 32],
}

impl Chunker {
    pub fn new(root_path: &Path) -> Result<Self, std::io::Error> {
        let mut files = Vec::new();
        let mut total_size = 0;
        let mut total_chunks = 0;

        if root_path.is_file() {
            let size = root_path.metadata()?.len();
            let chunks = if size == 0 {
                1
            } else {
                size.div_ceil(CHUNK_SIZE as u64)
            };
            files.push(FileEntry {
                path: root_path.to_path_buf(),
                relative_path: root_path.file_name().unwrap().to_string_lossy().to_string(),
                size,
                chunks,
            });
            total_size += size;
            total_chunks += chunks;
        } else {
            for entry in WalkDir::new(root_path).into_iter().filter_map(|e| e.ok()) {
                if entry.file_type().is_file() {
                    let size = entry.metadata()?.len();
                    let chunks = if size == 0 {
                        1
                    } else {
                        size.div_ceil(CHUNK_SIZE as u64)
                    };

                    let rel_path = entry.path().strip_prefix(root_path).unwrap();
                    let mut rel_str = rel_path.to_string_lossy().to_string();
                    let root_name = root_path.file_name().unwrap().to_string_lossy().to_string();
                    if rel_str.is_empty() {
                        rel_str = root_name;
                    } else {
                        rel_str = format!("{}/{}", root_name, rel_str).replace("\\", "/");
                    }

                    files.push(FileEntry {
                        path: entry.into_path(),
                        relative_path: rel_str,
                        size,
                        chunks,
                    });
                    total_size += size;
                    total_chunks += chunks;
                }
            }
        }

        files.sort_by(|a, b| a.relative_path.cmp(&b.relative_path));

        let mut blake3_hasher = blake3::Hasher::new();
        let mut chunk_hashes = Vec::with_capacity(total_chunks as usize);

        for file_entry in &files {
            let mut f = File::open(&file_entry.path)?;
            let mut buf = vec![0u8; CHUNK_SIZE];
            if file_entry.size == 0 {
                chunk_hashes.push(blake3::hash(&[]).into());
                blake3_hasher.update(&[]);
            } else {
                loop {
                    let n = f.read(&mut buf)?;
                    if n == 0 {
                        break;
                    }
                    let chunk_data = &buf[..n];
                    blake3_hasher.update(chunk_data);
                    chunk_hashes.push(blake3::hash(chunk_data).into());
                }
            }
        }

        if files.is_empty() {
            // Empty folder fallback
            chunk_hashes.push(blake3::hash(&[]).into());
            total_chunks = 1;
        }

        Ok(Self {
            files,
            total_size,
            total_chunks,
            chunk_hashes,
            overall_hash: blake3_hasher.finalize().into(),
        })
    }

    pub fn read_chunk(&self, index: u64) -> Result<Vec<u8>, std::io::Error> {
        if index >= self.total_chunks {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "Index out of bounds",
            ));
        }

        if self.files.is_empty() {
            return Ok(Vec::new());
        }

        let mut current_chunk_offset = 0;
        for file_entry in &self.files {
            if index < current_chunk_offset + file_entry.chunks {
                let file_chunk_index = index - current_chunk_offset;
                let mut f = File::open(&file_entry.path)?;
                f.seek(SeekFrom::Start(file_chunk_index * CHUNK_SIZE as u64))?;

                let expected_len = if file_chunk_index == file_entry.chunks - 1
                    && !file_entry.size.is_multiple_of(CHUNK_SIZE as u64)
                {
                    (file_entry.size % (CHUNK_SIZE as u64)) as usize
                } else if file_entry.size == 0 {
                    0
                } else {
                    CHUNK_SIZE
                };

                let mut buf = vec![0u8; expected_len];
                f.read_exact(&mut buf)?;
                return Ok(buf);
            }
            current_chunk_offset += file_entry.chunks;
        }

        Err(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "Chunk not mapped to any file",
        ))
    }

    pub fn total_chunks(&self) -> u64 {
        self.total_chunks
    }

    pub fn total_size(&self) -> u64 {
        self.total_size
    }

    pub fn blake3_hash(&self) -> [u8; 32] {
        self.overall_hash
    }

    pub fn chunk_hashes(&self) -> Vec<[u8; 32]> {
        self.chunk_hashes.clone()
    }

    pub fn get_metadata(&self) -> Vec<FileMetadata> {
        self.files
            .iter()
            .map(|f| FileMetadata {
                relative_path: f.relative_path.clone(),
                size: f.size,
                chunks: f.chunks,
            })
            .collect()
    }
}

use std::fs::OpenOptions;
use std::io::Write;

pub struct VirtualWriter {
    pub files: Vec<FileMetadata>,
    pub base_dir: PathBuf,
}

impl VirtualWriter {
    pub fn new(base_dir: &Path, files: Vec<FileMetadata>) -> Self {
        Self {
            base_dir: base_dir.to_path_buf(),
            files,
        }
    }

    pub fn write_chunk(&self, index: u64, data: &[u8]) -> Result<(), std::io::Error> {
        if self.files.is_empty() {
            return Ok(());
        }

        let mut current_chunk_offset = 0;
        for file_entry in &self.files {
            if index < current_chunk_offset + file_entry.chunks {
                let file_chunk_index = index - current_chunk_offset;
                let safe_rel = crate::types::safe_path(&file_entry.relative_path)
                    .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidInput, e.to_string()))?;
        let full_path = self.base_dir.join(&safe_rel);
                
                if let Some(parent) = full_path.parent() {
                    std::fs::create_dir_all(parent)?;
                }

                let mut f = OpenOptions::new()
                    .read(true)
                    .write(true)
                    .create(true)
                    .truncate(false)
                    .open(&full_path)?;
                    
                f.seek(SeekFrom::Start(file_chunk_index * CHUNK_SIZE as u64))?;
                f.write_all(data)?;
                return Ok(());
            }
            current_chunk_offset += file_entry.chunks;
        }

        Err(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "Chunk not mapped to any file",
        ))
    }
    
    pub fn verify_chunk(&self, index: u64, expected_hash: &[u8; 32]) -> Result<bool, std::io::Error> {
        if self.files.is_empty() {
            let hash = blake3::hash(&[]);
            return Ok(hash.as_bytes() == expected_hash);
        }
        
        let mut current_chunk_offset = 0;
        for file_entry in &self.files {
            if index < current_chunk_offset + file_entry.chunks {
                let file_chunk_index = index - current_chunk_offset;
                let safe_rel = crate::types::safe_path(&file_entry.relative_path)
                    .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidInput, e.to_string()))?;
                let full_path = self.base_dir.join(&safe_rel);
                
                if !full_path.exists() {
                    return Ok(false);
                }

                let mut f = File::open(&full_path)?;
                f.seek(SeekFrom::Start(file_chunk_index * CHUNK_SIZE as u64))?;

                let expected_len = if file_chunk_index == file_entry.chunks - 1
                    && !file_entry.size.is_multiple_of(CHUNK_SIZE as u64)
                {
                    (file_entry.size % (CHUNK_SIZE as u64)) as usize
                } else if file_entry.size == 0 {
                    0
                } else {
                    CHUNK_SIZE
                };

                let mut buf = vec![0u8; expected_len];
                match f.read_exact(&mut buf) {
                    Ok(_) => {
                        let hash = blake3::hash(&buf);
                        return Ok(hash.as_bytes() == expected_hash);
                    }
                    Err(_) => return Ok(false),
                }
            }
            current_chunk_offset += file_entry.chunks;
        }
        Ok(false)
    }

    pub fn blake3_hash(&self) -> [u8; 32] {
        let mut hasher = blake3::Hasher::new();
        for file_entry in &self.files {
            let safe_rel = crate::types::safe_path(&file_entry.relative_path).unwrap_or_default();
            let full_path = self.base_dir.join(&safe_rel);
            if let Ok(mut f) = std::fs::File::open(&full_path) {
                let _ = std::io::copy(&mut f, &mut hasher);
            }
        }
        hasher.finalize().into()
    }
}
