#[derive(Debug, thiserror::Error)]
pub enum DropWireError {
    #[error("invalid code phrase: {0}")]
    InvalidCodePhrase(String),
    #[error("crypto error: {0}")]
    Crypto(String),
    #[error("network error: {0}")]
    Network(String),
    #[error("protocol error: {0}")]
    Protocol(String),
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("transfer cancelled")]
    Cancelled,
    #[error("peer disconnected")]
    PeerDisconnected,
    #[error("hash mismatch")]
    HashMismatch,
    #[error("timeout")]
    Timeout,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_implements_std_error() {
        let err = DropWireError::Cancelled;
        let _dyn_err: &dyn std::error::Error = &err;
    }
}
