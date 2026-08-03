use crate::error::DropWireError;
use chacha20poly1305::{
    aead::{Aead, KeyInit},
    ChaCha20Poly1305, Key, Nonce,
};

use chacha20poly1305::aead::Payload;

pub fn encrypt_chunk(
    stream_key: &[u8; 32],
    chunk_idx: u64,
    plaintext: &[u8],
) -> Result<Vec<u8>, DropWireError> {
    let key = Key::from_slice(stream_key);
    let cipher = ChaCha20Poly1305::new(key);

    let mut nonce_bytes = [0u8; 12];
    let idx_bytes = chunk_idx.to_be_bytes();
    nonce_bytes[4..12].copy_from_slice(&idx_bytes);
    let nonce = Nonce::from_slice(&nonce_bytes);

    let payload = Payload {
        msg: plaintext,
        aad: &idx_bytes,
    };

    cipher
        .encrypt(nonce, payload)
        .map_err(|_| DropWireError::Crypto("Encryption failed".into()))
}

pub fn decrypt_chunk(
    stream_key: &[u8; 32],
    chunk_idx: u64,
    ciphertext: &[u8],
) -> Result<Vec<u8>, DropWireError> {
    let key = Key::from_slice(stream_key);
    let cipher = ChaCha20Poly1305::new(key);

    let mut nonce_bytes = [0u8; 12];
    let idx_bytes = chunk_idx.to_be_bytes();
    nonce_bytes[4..12].copy_from_slice(&idx_bytes);
    let nonce = Nonce::from_slice(&nonce_bytes);

    let payload = Payload {
        msg: ciphertext,
        aad: &idx_bytes,
    };

    cipher
        .decrypt(nonce, payload)
        .map_err(|_| DropWireError::Crypto("Decryption failed".into()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_roundtrip() {
        let key = [42u8; 32];
        let plaintext = b"Hello DropWire Protocol";

        let ciphertext = encrypt_chunk(&key, 1, plaintext).unwrap();
        let decrypted = decrypt_chunk(&key, 1, &ciphertext).unwrap();
        assert_eq!(plaintext.as_slice(), decrypted.as_slice());
    }

    #[test]
    fn test_tamper() {
        let key = [42u8; 32];
        let plaintext = b"Hello DropWire Protocol";

        let mut ciphertext = encrypt_chunk(&key, 1, plaintext).unwrap();

        // Tamper with the ciphertext
        let len = ciphertext.len();
        ciphertext[len - 1] ^= 0x01;

        let result = decrypt_chunk(&key, 1, &ciphertext);
        assert!(matches!(result, Err(DropWireError::Crypto(_))));
    }

    #[test]
    fn test_wrong_key() {
        let key = [42u8; 32];
        let wrong_key = [43u8; 32];
        let plaintext = b"Hello DropWire Protocol";

        let ciphertext = encrypt_chunk(&key, 1, plaintext).unwrap();

        let result = decrypt_chunk(&wrong_key, 1, &ciphertext);
        assert!(matches!(result, Err(DropWireError::Crypto(_))));
    }

    #[test]
    fn test_wrong_nonce() {
        let key = [42u8; 32];
        let plaintext = b"Hello DropWire Protocol";

        let ciphertext = encrypt_chunk(&key, 1, plaintext).unwrap();

        let result = decrypt_chunk(&key, 2, &ciphertext);
        assert!(matches!(result, Err(DropWireError::Crypto(_))));
    }
}
