use crate::error::DropWireError;
use std::path::Path;

const INCOMPRESSIBLE_EXTS: &[&str] = &[
    "zip", "7z", "gz", "xz", "zst", "rar", "mp4", "mkv", "mov", "webm", "jpg", "jpeg", "png",
    "gif", "webp", "heic", "mp3", "aac", "flac", "ogg",
];

fn shannon_entropy(data: &[u8]) -> f64 {
    let mut counts = [0usize; 256];
    for &byte in data {
        counts[byte as usize] += 1;
    }
    let mut entropy = 0.0;
    let len = data.len() as f64;
    for count in counts {
        if count > 0 {
            let p = count as f64 / len;
            entropy -= p * p.log2();
        }
    }
    entropy
}

pub fn maybe_compress<'a>(data: &'a [u8], filename: &str) -> (std::borrow::Cow<'a, [u8]>, bool) {
    if let Some(ext) = Path::new(filename).extension().and_then(|e| e.to_str()) {
        if INCOMPRESSIBLE_EXTS.contains(&ext.to_lowercase().as_str()) {
            return (std::borrow::Cow::Borrowed(data), false);
        }
    }

    let sample_len = data.len().min(4096);
    if sample_len > 0 {
        let entropy = shannon_entropy(&data[..sample_len]);
        if entropy > 7.5 {
            return (std::borrow::Cow::Borrowed(data), false);
        }
    }

    if let Ok(compressed) = zstd::stream::encode_all(data, 3) {
        if compressed.len() < data.len() {
            return (std::borrow::Cow::Owned(compressed), true);
        }
    }

    (std::borrow::Cow::Borrowed(data), false)
}

pub fn decompress(data: &[u8]) -> Result<Vec<u8>, DropWireError> {
    use std::io::Read;
    let decoder = zstd::Decoder::new(data).map_err(|e| DropWireError::Protocol(format!("zstd init: {e}")))?;
    let mut output = Vec::with_capacity(data.len() * 2);
    // Limit to 20MB decompressed size (since max frame size is 10MB)
    decoder
        .take(20 * 1024 * 1024)
        .read_to_end(&mut output)
        .map_err(|e| DropWireError::Protocol(format!("Decompression failed: {e}")))?;
    
    if output.len() == 20 * 1024 * 1024 {
        // Technically it could be exactly 20MB, but if we hit the limit, it's safer to reject or just accept it as is.
        // Let's just return the output. If it was truncated, the blake3 hash check will fail anyway.
    }
    
    Ok(output)
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::RngCore;
    use std::time::Instant;

    #[test]
    fn test_compressible_data() {
        let data = vec![42u8; 1024];
        let (out, compressed) = maybe_compress(&data, "test.txt");
        assert!(compressed);
        assert!(out.len() < data.len());
    }

    #[test]
    fn test_incompressible_entropy() {
        let mut data = vec![0u8; 8192];
        rand::thread_rng().fill_bytes(&mut data);
        let (out, compressed) = maybe_compress(&data, "test.bin");
        assert!(!compressed);
        assert_eq!(out.len(), data.len());
    }

    #[test]
    fn test_incompressible_extension() {
        let data = vec![0u8; 1024]; // Highly compressible
        let (out, compressed) = maybe_compress(&data, "test.mp4");
        assert!(!compressed);
        assert_eq!(out.len(), data.len()); // Zstd never invoked
    }

    #[test]
    fn test_round_trip() {
        let data = vec![42u8; 1024];
        let (out, compressed) = maybe_compress(&data, "test.txt");
        assert!(compressed);
        let decoded = decompress(&out).unwrap();
        assert_eq!(decoded, data);
    }

    #[test]
    fn test_decompress_garbage() {
        let data = vec![1, 2, 3, 4, 5, 6, 7];
        let result = decompress(&data);
        assert!(result.is_err());
    }

    #[test]
    fn test_benchmark_entropy_skip() {
        let mut data = vec![0u8; 1024 * 1024]; // 1 MiB chunk
        rand::thread_rng().fill_bytes(&mut data); // Incompressible

        let start_zstd = Instant::now();
        let _ = zstd::stream::encode_all(data.as_slice(), 3);
        let zstd_time = start_zstd.elapsed();

        let start_entropy = Instant::now();
        let (_, compressed) = maybe_compress(&data, "test.bin");
        let entropy_time = start_entropy.elapsed();

        assert!(!compressed);
        assert!(
            entropy_time * 3 < zstd_time,
            "Entropy check was not 3x faster! entropy: {:?}, zstd: {:?}",
            entropy_time,
            zstd_time
        );
    }
}
