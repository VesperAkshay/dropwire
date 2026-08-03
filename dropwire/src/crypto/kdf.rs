pub fn derive_stream_key(master: &[u8; 32], stream_id: u32) -> [u8; 32] {
    let context = format!("dropwire/stream/{:04x}", stream_id);
    blake3::derive_key(&context, master)
}

pub fn derive_auth_token(master: &[u8; 32]) -> [u8; 16] {
    let context = "dropwire/auth/v1";
    let key = blake3::derive_key(context, master);
    let mut token = [0u8; 16];
    token.copy_from_slice(&key[0..16]);
    token
}

pub fn derive_control_key(master: &[u8; 32]) -> [u8; 32] {
    let context = "dropwire/control/v1";
    blake3::derive_key(context, master)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_same_inputs_same_outputs() {
        let master = [1u8; 32];
        let stream_key_1 = derive_stream_key(&master, 0);
        let stream_key_2 = derive_stream_key(&master, 0);
        assert_eq!(stream_key_1, stream_key_2);

        let auth_1 = derive_auth_token(&master);
        let auth_2 = derive_auth_token(&master);
        assert_eq!(auth_1, auth_2);

        let ctl_1 = derive_control_key(&master);
        let ctl_2 = derive_control_key(&master);
        assert_eq!(ctl_1, ctl_2);
    }

    #[test]
    fn test_different_stream_ids_different_keys() {
        let master = [2u8; 32];
        let stream_key_0 = derive_stream_key(&master, 0);
        let stream_key_1 = derive_stream_key(&master, 1);
        assert_ne!(stream_key_0, stream_key_1);
    }

    #[test]
    fn test_output_lengths() {
        // Output lengths are implicitly verified by return types [u8; 32] and [u8; 16]
        let master = [3u8; 32];
        let stream_key = derive_stream_key(&master, 0);
        let auth_token = derive_auth_token(&master);
        assert_eq!(stream_key.len(), 32);
        assert_eq!(auth_token.len(), 16);
    }
}
