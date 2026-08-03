use crate::error::DropWireError;
use crate::types::FRAME_MAX_SIZE;
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};

pub async fn write_frame<W: AsyncWrite + Unpin>(
    writer: &mut W,
    payload: &[u8],
) -> Result<(), DropWireError> {
    if payload.len() > FRAME_MAX_SIZE as usize {
        return Err(DropWireError::Protocol("frame too large".into()));
    }

    let len = payload.len() as u32;
    writer
        .write_all(&len.to_be_bytes())
        .await
        .map_err(DropWireError::Io)?;
    writer.write_all(payload).await.map_err(DropWireError::Io)?;
    Ok(())
}

pub async fn read_frame<R: AsyncRead + Unpin>(reader: &mut R) -> Result<Vec<u8>, DropWireError> {
    let mut len_bytes = [0u8; 4];
    match reader.read_exact(&mut len_bytes).await {
        Ok(_) => {}
        Err(e) if e.kind() == std::io::ErrorKind::UnexpectedEof => {
            return Err(DropWireError::PeerDisconnected);
        }
        Err(e) => return Err(DropWireError::Io(e)),
    }

    let len = u32::from_be_bytes(len_bytes);
    if len > FRAME_MAX_SIZE {
        return Err(DropWireError::Protocol("frame too large".into()));
    }

    let mut payload = vec![0u8; len as usize];
    match reader.read_exact(&mut payload).await {
        Ok(_) => Ok(payload),
        Err(e) if e.kind() == std::io::ErrorKind::UnexpectedEof => {
            Err(DropWireError::PeerDisconnected)
        }
        Err(e) => Err(DropWireError::Io(e)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::io::duplex;

    #[tokio::test]
    async fn test_roundtrip() {
        let (mut client, mut server) = duplex(1024);
        let payload = b"Hello Framing Protocol";

        write_frame(&mut client, payload).await.unwrap();
        let received = read_frame(&mut server).await.unwrap();

        assert_eq!(payload.as_slice(), received.as_slice());
    }

    #[tokio::test]
    async fn test_empty_payload() {
        let (mut client, mut server) = duplex(1024);
        let payload = b"";

        write_frame(&mut client, payload).await.unwrap();
        let received = read_frame(&mut server).await.unwrap();

        assert_eq!(payload.as_slice(), received.as_slice());
    }

    #[tokio::test]
    async fn test_1mib_payload() {
        // Need a buffer large enough for 1MB payload + 4 byte header
        let (mut client, mut server) = duplex(2 * 1024 * 1024);
        let payload = vec![0x42u8; 1024 * 1024];

        // Spawning writer so it doesn't deadlock the duplex stream if buffer is tight
        let payload_clone = payload.clone();
        tokio::spawn(async move {
            write_frame(&mut client, &payload_clone).await.unwrap();
        });

        let received = read_frame(&mut server).await.unwrap();
        assert_eq!(payload, received);
    }

    #[tokio::test]
    async fn test_oversized_payload_write() {
        let (mut client, _) = duplex(1024);
        // Oversized payload (17 MiB)
        let payload = vec![0x00u8; 17 * 1024 * 1024];

        let result = write_frame(&mut client, &payload).await;
        assert!(matches!(result, Err(DropWireError::Protocol(_))));
    }

    #[tokio::test]
    async fn test_oversized_payload_read() {
        let (mut client, mut server) = duplex(1024);
        let mut oversize_header = Vec::new();
        oversize_header.extend_from_slice(&(FRAME_MAX_SIZE + 1).to_be_bytes());

        client.write_all(&oversize_header).await.unwrap();

        let result = read_frame(&mut server).await;
        assert!(matches!(result, Err(DropWireError::Protocol(_))));
    }

    #[tokio::test]
    async fn test_truncated_stream() {
        let (mut client, mut server) = duplex(1024);

        // Write valid header but no body
        let header = 100u32.to_be_bytes();
        client.write_all(&header).await.unwrap();
        drop(client); // close stream mid-read

        let result = read_frame(&mut server).await;
        assert!(matches!(result, Err(DropWireError::PeerDisconnected)));
    }
}
