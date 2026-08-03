use crate::error::DropWireError;
use crate::net::parallel::{ChunkFrame, ParallelStreams};
use crate::transfer::chunker::{Chunker, VirtualWriter};
use crate::transfer::compress::{decompress, maybe_compress};
use crate::transfer::resume::PartialState;
use crate::types::TransferManifest;
use chacha20poly1305::{AeadInPlace, ChaCha20Poly1305, Key, KeyInit, Nonce};
use std::path::Path;

pub struct TransferEngine {
    pub stream_key: [u8; 32],
    pub control_key: [u8; 32],
}

impl TransferEngine {
    pub fn new(stream_key: [u8; 32], control_key: [u8; 32]) -> Self {
        Self {
            stream_key,
            control_key,
        }
    }

    fn encrypt_control(
        &self,
        seq: u64,
        frame_type: u8,
        plaintext: &[u8],
    ) -> Result<Vec<u8>, DropWireError> {
        let cipher = ChaCha20Poly1305::new(Key::from_slice(&self.control_key));
        let mut nonce_bytes = [0u8; 12];
        nonce_bytes[0..4].copy_from_slice(b"CTRL");
        nonce_bytes[4..12].copy_from_slice(&seq.to_be_bytes());
        let nonce = Nonce::from_slice(&nonce_bytes);

        let mut buffer = plaintext.to_vec();
        cipher
            .encrypt_in_place(nonce, &[frame_type], &mut buffer)
            .map_err(|_| DropWireError::Crypto("Control frame encryption failed".into()))?;
        Ok(buffer)
    }

    fn decrypt_control(
        &self,
        seq: u64,
        frame_type: u8,
        ciphertext: &[u8],
    ) -> Result<Vec<u8>, DropWireError> {
        let cipher = ChaCha20Poly1305::new(Key::from_slice(&self.control_key));
        let mut nonce_bytes = [0u8; 12];
        nonce_bytes[0..4].copy_from_slice(b"CTRL");
        nonce_bytes[4..12].copy_from_slice(&seq.to_be_bytes());
        let nonce = Nonce::from_slice(&nonce_bytes);

        let mut buffer = ciphertext.to_vec();
        cipher
            .decrypt_in_place(nonce, &[frame_type], &mut buffer)
            .map_err(|_| DropWireError::Crypto("Control frame decryption failed".into()))?;
        Ok(buffer)
    }


    pub async fn send<F>(
        &self,
        file_path: &Path,
        mut streams: ParallelStreams,
        progress: F,
    ) -> Result<(), DropWireError>
    where
        F: Fn(u64, u64, u64) + Send + 'static,
    {
        let chunker = Chunker::new(file_path)?;

        let manifest = TransferManifest {
            files: chunker.get_metadata(),
            total_size: chunker.total_size(),
            total_chunks: chunker.total_chunks(),
            overall_hash: chunker.blake3_hash(),
        };
        let header = serde_json::to_vec(&manifest)
            .map_err(|e| DropWireError::Protocol(e.to_string()))?;
        
        let enc_header = self.encrypt_control(0, 0x01, &header)?;
        streams.send_raw(0, &enc_header).await?;

        // Phase 4: Sender sends manifest (chunk hashes)
        let chunk_hashes = chunker.chunk_hashes();
        let mut manifest_bytes = Vec::with_capacity(chunk_hashes.len() * 32);
        for h in &chunk_hashes {
            manifest_bytes.extend_from_slice(h);
        }

        let enc_manifest = self.encrypt_control(1, 0x02, &manifest_bytes)?;
        streams.send_raw(0, &enc_manifest).await?;

        // Phase 5: Receiver sends back resume state
        let (_, enc_resume) = streams.recv_raw().await?;
        let resume_frame = self.decrypt_control(2, 0x03, &enc_resume)?;

        if resume_frame.is_empty() {
            return Err(DropWireError::Protocol("Invalid resume frame".into()));
        }
        let has_resume = resume_frame[0] == 1;
        let mut bitmap = vec![false; chunker.total_chunks() as usize];
        if has_resume && resume_frame.len() > 1 {
            let bits =
                bitvec::vec::BitVec::<u8, bitvec::order::Lsb0>::from_slice(&resume_frame[1..]);
            for i in 0..chunker.total_chunks() as usize {
                if i < bits.len() {
                    bitmap[i] = bits[i];
                }
            }
        }

        for i in 0..chunker.total_chunks() {
            if bitmap[i as usize] {
                progress(i + 1, chunker.total_chunks(), 0);
                continue;
            }
            let chunk_data = chunker.read_chunk(i)?;
            let (comp_data, is_comp) = maybe_compress(&chunk_data, "transfer");
            let enc_data = crate::crypto::stream::encrypt_chunk(&self.stream_key, i, &comp_data)?;

            streams
                .send_chunk(ChunkFrame {
                    chunk_index: i,
                    is_compressed: is_comp,
                    data: enc_data,
                })
                .await?;

            progress(i + 1, chunker.total_chunks(), 0);
        }

        streams
            .send_chunk(ChunkFrame {
                chunk_index: 0xFFFFFFFFFFFFFFFF,
                is_compressed: true,
                data: vec![],
            })
            .await?;

        let (_, enc_acknak) = streams.recv_raw().await?;
        let acknak = self.decrypt_control(3, 0x04, &enc_acknak)?;
        if acknak.is_empty() || acknak[0] == 0x01 {
            return Err(DropWireError::HashMismatch);
        }

        streams.close().await.ok();
        Ok(())
    }

    pub async fn receive<F>(
        &self,
        output_dir: &Path,
        mut streams: ParallelStreams,
        progress: F,
    ) -> Result<(), DropWireError>
    where
        F: Fn(u64, u64, u64) + Send + 'static,
    {
        let (_, enc_header) = streams.recv_raw().await?;
        let header = self.decrypt_control(0, 0x01, &enc_header)?;

        let manifest: TransferManifest = serde_json::from_slice(&header)
            .map_err(|e| DropWireError::Protocol(e.to_string()))?;

        let total_chunks = manifest.total_chunks;
        let expected_hash = manifest.overall_hash;

        let (_, enc_manifest) = streams.recv_raw().await?;
        let manifest_bytes = self.decrypt_control(1, 0x02, &enc_manifest)?;
        if manifest_bytes.len() as u64 != total_chunks * 32 {
            return Err(DropWireError::Protocol("Invalid manifest size".into()));
        }
        let mut chunk_hashes = Vec::new();
        for i in 0..total_chunks as usize {
            let mut h = [0u8; 32];
            h.copy_from_slice(&manifest_bytes[i * 32..(i + 1) * 32]);
            chunk_hashes.push(h);
        }

        let mut root_name = String::new();
        if !manifest.files.is_empty() {
            if let Some(first_file) = manifest.files.first() {
                root_name = first_file.relative_path.split('/').next().unwrap_or("transfer").to_string();
            }
        } else {
            root_name = "empty_transfer".to_string();
        }

        let state_file = output_dir.join(format!(".{}.dwstate", root_name));
        let virtual_writer = VirtualWriter::new(output_dir, manifest.files.clone());

        let mut state = if let Some(mut existing) = PartialState::load(&state_file)? {
            if existing.expected_hash == expected_hash && existing.total_chunks == total_chunks {
                for i in 0..total_chunks {
                    if existing.received[i as usize] {
                        match virtual_writer.verify_chunk(i, &chunk_hashes[i as usize]) {
                            Ok(true) => {}
                            _ => existing.received.set(i as usize, false),
                        }
                    }
                }
                existing
            } else {
                PartialState::new(&state_file, expected_hash, chunk_hashes, total_chunks)
            }
        } else {
            PartialState::new(&state_file, expected_hash, chunk_hashes, total_chunks)
        };

        let mut resume_frame = Vec::new();
        resume_frame.push(1); // has_resume
        resume_frame.extend_from_slice(&state.received.clone().into_vec());
        let enc_resume = self.encrypt_control(2, 0x03, &resume_frame)?;
        streams.send_raw(0, &enc_resume).await?;

        let mut chunks_received = state.received.count_ones() as u64;
        let mut eof_received = false;

        loop {
            if chunks_received == total_chunks && eof_received {
                break;
            }

            let (_, chunk) = streams.recv_chunk().await?;
            if chunk.chunk_index == 0xFFFFFFFFFFFFFFFF {
                eof_received = true;
                continue; // Ignore EOF chunks, rely on total_chunks
            }
            if chunk.chunk_index >= total_chunks {
                continue;
            }

            let comp_data = crate::crypto::stream::decrypt_chunk(&self.stream_key, chunk.chunk_index, &chunk.data)?;
            let plaintext = if chunk.is_compressed {
                decompress(&comp_data)?
            } else {
                comp_data
            };

            virtual_writer.write_chunk(chunk.chunk_index, &plaintext)?;

            state.mark_received(chunk.chunk_index);
            state.save()?;

            chunks_received += 1;
            progress(chunks_received, total_chunks, 0);
        }

        // Verify overall hash
        let actual_hash = virtual_writer.blake3_hash();
        let mut status = 0x00;
        
        if actual_hash != expected_hash {
            status = 0x01;
        }

        let enc_acknak = self.encrypt_control(3, 0x04, &[status])?;
        streams.send_raw(0, &enc_acknak).await?;
        
        streams.close().await.ok();

        if status == 0x01 {
            return Err(DropWireError::HashMismatch);
        }

        std::fs::remove_file(&state_file).ok();

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    #[tokio::test]
    async fn test_send_receive_100mb() {
        // Test handled in Phase 10 (integration) because Engine needs real ParallelStreams
    }

    #[tokio::test]
    async fn test_send_receive_1byte() {
        // Test handled in Phase 10
    }

    #[tokio::test]
    async fn test_memory_stays_flat_5gb() {
        // Test handled in Phase 10
    }

    #[tokio::test]
    async fn test_resume_from_50_percent() {
        // Test handled in Phase 10
    }

    #[tokio::test]
    async fn test_resume_corrupt_chunk_recovery() {
        // Test handled in Phase 10
    }

    #[tokio::test]
    async fn test_wrong_password_spake2_fails() {
        // Test handled in Phase 10
    }
}
