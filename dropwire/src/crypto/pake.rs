use crate::error::DropWireError;
use crate::types::SharedSecret;
use spake2::{Ed25519Group, Identity, Password, Spake2};

pub struct PakeState {
    inner: Spake2<Ed25519Group>,
    pub outbound_msg: [u8; 32],
    is_alice: bool,
}

impl PakeState {
    pub fn new(password: &str, channel: &str, is_alice: bool) -> (Self, [u8; 32]) {
        let mut combined = Vec::new();
        combined.extend_from_slice(password.as_bytes());
        combined.extend_from_slice(channel.as_bytes());

        let pwd = Password::new(&combined);
        let id_a = Identity::new(b"dropwire-sender");
        let id_b = Identity::new(b"dropwire-receiver");

        let (inner, outbound) = if is_alice {
            Spake2::<Ed25519Group>::start_a(&pwd, &id_a, &id_b)
        } else {
            Spake2::<Ed25519Group>::start_b(&pwd, &id_a, &id_b)
        };

        let mut msg_arr = [0u8; 32];
        msg_arr.copy_from_slice(&outbound[1..33]);

        (
            Self {
                inner,
                outbound_msg: msg_arr,
                is_alice,
            },
            msg_arr,
        )
    }

    pub fn finish(self, peer_msg: &[u8; 32]) -> Result<SharedSecret, DropWireError> {
        let is_zeros = peer_msg.iter().all(|&b| b == 0);
        let mut id = [0u8; 32];
        id[0] = 1;

        if is_zeros || peer_msg == &id {
            return Err(DropWireError::Crypto("Degenerate peer message".into()));
        }

        let mut full_peer_msg = Vec::with_capacity(33);
        if self.is_alice {
            full_peer_msg.push(b'B');
        } else {
            full_peer_msg.push(b'A');
        }
        full_peer_msg.extend_from_slice(peer_msg);

        let raw_secret = self
            .inner
            .finish(&full_peer_msg)
            .map_err(|_| DropWireError::Crypto("SPAKE2 finish failed".into()))?;

        let mut hasher = blake3::Hasher::new();
        hasher.update(&raw_secret);
        hasher.update(b"dropwire-sender");
        hasher.update(b"dropwire-receiver");

        if self.is_alice {
            hasher.update(&self.outbound_msg);
            hasher.update(peer_msg);
        } else {
            hasher.update(peer_msg);
            hasher.update(&self.outbound_msg);
        }

        let k = hasher.finalize();
        let mut key = [0u8; 32];
        key.copy_from_slice(k.as_bytes());
        Ok(SharedSecret(key))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    #[test]
    fn test_happy_path() {
        let (alice, alice_msg) = PakeState::new("7-guitar-abandon", "7", true);
        let (bob, bob_msg) = PakeState::new("7-guitar-abandon", "7", false);

        let alice_key = alice.finish(&bob_msg).unwrap();
        let bob_key = bob.finish(&alice_msg).unwrap();

        assert_eq!(alice_key.0, bob_key.0);
    }

    #[test]
    fn test_wrong_password() {
        let (alice, alice_msg) = PakeState::new("7-guitar-abandon", "7", true);
        let (bob, bob_msg) = PakeState::new("7-different-word", "7", false);

        let alice_key = alice.finish(&bob_msg);
        let bob_key = bob.finish(&alice_msg);

        if let (Ok(ak), Ok(bk)) = (alice_key, bob_key) {
            assert_ne!(ak.0, bk.0);
        }
    }

    #[test]
    fn test_identity_point_rejection() {
        let (alice, _) = PakeState::new("7-guitar-abandon", "7", true);

        let zeros = [0u8; 32];
        let res_zeros = alice.finish(&zeros);
        assert!(matches!(res_zeros, Err(DropWireError::Crypto(_))));

        let (alice2, _) = PakeState::new("7-guitar-abandon", "7", true);
        let mut identity = [0u8; 32];
        identity[0] = 1;
        let res_id = alice2.finish(&identity);
        assert!(matches!(res_id, Err(DropWireError::Crypto(_))));
    }

    proptest! {
        #[test]
        fn test_property_random_passwords(password in "[a-zA-Z0-9-]{10,50}") {
            let channel = "123";
            let (alice, alice_msg) = PakeState::new(&password, channel, true);
            let (bob, bob_msg) = PakeState::new(&password, channel, false);

            let alice_key = alice.finish(&bob_msg).unwrap();
            let bob_key = bob.finish(&alice_msg).unwrap();

            assert_eq!(alice_key.0, bob_key.0);
        }
    }
}
