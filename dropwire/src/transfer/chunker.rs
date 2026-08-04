use crate::types::{FileMetadata, CHUNK_SIZE};
use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::path::{Path, PathBuf};
use ignore::WalkBuilder;

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
    pub chunk_size: u32,
    file_cache: std::sync::Mutex<Option<(PathBuf, File)>>,
}

impl Chunker {
    pub fn new(root_path: &Path, chunk_size: u32) -> Result<Self, std::io::Error> {
        Self::from_paths(&[root_path.to_path_buf()], chunk_size)
    }

    pub fn from_paths(paths: &[PathBuf], chunk_size: u32) -> Result<Self, std::io::Error> {
        let mut files = Vec::new();
        let mut total_size = 0;
        let mut total_chunks = 0;

        for root_path in paths {
            let base_parent = root_path.parent().unwrap_or(root_path);
            
            if root_path.is_file() {
                let size = root_path.metadata()?.len();
                let chunks = if size == 0 {
                    1
                } else {
                    size.div_ceil(chunk_size as u64)
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
                for entry in WalkBuilder::new(root_path).hidden(false).require_git(false).build().filter_map(|e| e.ok()) {
                    if entry.file_type().map_or(false, |ft| ft.is_file()) {
                        let size = entry.metadata().map(|m| m.len()).unwrap_or(0);
                        let chunks = if size == 0 {
                            1
                        } else {
                            size.div_ceil(CHUNK_SIZE as u64)
                        };

                        let rel_path = entry.path().strip_prefix(base_parent).unwrap_or(entry.path());
                        let rel_str = rel_path.to_string_lossy().to_string().replace("\\", "/");

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
        }

        files.sort_by(|a, b| a.relative_path.cmp(&b.relative_path));

        let mut blake3_hasher = blake3::Hasher::new();
        let mut chunk_hashes = Vec::with_capacity(total_chunks as usize);

        for file_entry in &files {
            let mut f = File::open(&file_entry.path)?;
            let mut buf = vec![0u8; chunk_size as usize];
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
            chunk_size,
            file_cache: std::sync::Mutex::new(None),
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
                
                let mut cache = self.file_cache.lock().unwrap();
                let mut f = if let Some((path, file)) = cache.take() {
                    if path == file_entry.path {
                        file
                    } else {
                        File::open(&file_entry.path)?
                    }
                } else {
                    File::open(&file_entry.path)?
                };
                
                f.seek(SeekFrom::Start(file_chunk_index * self.chunk_size as u64))?;

                let expected_len = if file_chunk_index == file_entry.chunks - 1
                    && !file_entry.size.is_multiple_of(self.chunk_size as u64)
                {
                    (file_entry.size % (self.chunk_size as u64)) as usize
                } else if file_entry.size == 0 {
                    0
                } else {
                    self.chunk_size as usize
                };

                let mut buf = vec![0u8; expected_len];
                f.read_exact(&mut buf)?;
                
                *cache = Some((file_entry.path.clone(), f));
                
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
    pub chunk_size: u32,
    file_cache: std::sync::Mutex<Option<(PathBuf, File)>>,
}

impl VirtualWriter {
    pub fn new(base_dir: &Path, files: Vec<FileMetadata>, chunk_size: u32) -> Self {
        Self {
            base_dir: base_dir.to_path_buf(),
            files,
            chunk_size,
            file_cache: std::sync::Mutex::new(None),
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

                let mut cache = self.file_cache.lock().unwrap();
                let mut f = if let Some((path, file)) = cache.take() {
                    if path == full_path {
                        file
                    } else {
                        OpenOptions::new().read(true).write(true).create(true).truncate(false).open(&full_path)?
                    }
                } else {
                    OpenOptions::new().read(true).write(true).create(true).truncate(false).open(&full_path)?
                };
                    
                f.seek(SeekFrom::Start(file_chunk_index * self.chunk_size as u64))?;
                f.write_all(data)?;
                
                *cache = Some((full_path, f));
                
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
                f.seek(SeekFrom::Start(file_chunk_index * self.chunk_size as u64))?;

                let expected_len = if file_chunk_index == file_entry.chunks - 1
                    && !file_entry.size.is_multiple_of(self.chunk_size as u64)
                {
                    (file_entry.size % (self.chunk_size as u64)) as usize
                } else if file_entry.size == 0 {
                    0
                } else {
                    self.chunk_size as usize
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_chunker_respects_gitignore() {
        let temp_dir = tempfile::tempdir().unwrap();
        let root = temp_dir.path();

        // 1. Create a normal file (should be included)
        fs::write(root.join("keep.txt"), b"keep me").unwrap();

        // 2. Create a .gitignore file
        fs::write(root.join(".gitignore"), b"target/\nignored.txt\n").unwrap();

        // 3. Create a file that should be explicitly ignored
        fs::write(root.join("ignored.txt"), b"ignore me").unwrap();

        // 4. Create a directory that should be ignored, with a file inside it
        fs::create_dir(root.join("target")).unwrap();
        fs::write(root.join("target").join("built.exe"), b"binary").unwrap();

        // 5. Run the chunker
        let chunker = Chunker::new(root, 1024 * 1024).unwrap();
        let metadata = chunker.get_metadata();

        // Extract the relative paths that were picked up
        let paths: Vec<String> = metadata.into_iter().map(|m| m.relative_path).collect();

        // Assertions
        assert!(paths.iter().any(|p| p.contains("keep.txt")), "keep.txt should be included");
        assert!(paths.iter().any(|p| p.contains(".gitignore")), ".gitignore itself should be included");
        
        assert!(!paths.iter().any(|p| p.contains("ignored.txt")), "ignored.txt should be ignored");
        assert!(!paths.iter().any(|p| p.contains("built.exe")), "target/built.exe should be ignored");
    }
}
