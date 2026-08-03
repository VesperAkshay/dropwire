use crate::error::DropWireError;
use crate::net::signaling::Role;
use crate::types::ChannelId;
use std::net::SocketAddr;

pub struct ChunkFrame {
    pub chunk_index: u64,
    pub is_compressed: bool,
    pub data: Vec<u8>,
}

impl ChunkFrame {
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut buf = Vec::with_capacity(13 + self.data.len());
        buf.extend_from_slice(&self.chunk_index.to_be_bytes());
        buf.push(if self.is_compressed { 1 } else { 0 });
        buf.extend_from_slice(&(self.data.len() as u32).to_be_bytes());
        buf.extend_from_slice(&self.data);
        buf
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<Self, DropWireError> {
        if bytes.len() < 13 {
            return Err(DropWireError::Protocol("Frame too short".into()));
        }
        let chunk_index = u64::from_be_bytes(bytes[0..8].try_into().unwrap());
        let is_compressed = bytes[8] == 1;
        let data_len = u32::from_be_bytes(bytes[9..13].try_into().unwrap()) as usize;
        if bytes.len() < 13 + data_len {
            return Err(DropWireError::Protocol("Frame data too short".into()));
        }
        let data = bytes[13..13 + data_len].to_vec();
        Ok(Self {
            chunk_index,
            is_compressed,
            data,
        })
    }
}

pub struct FramedStream {
    pub write: tokio::net::tcp::OwnedWriteHalf,
}

pub struct ParallelStreams {
    pub streams: Vec<FramedStream>,
    #[allow(clippy::type_complexity)]
    rx: Option<tokio::sync::mpsc::Receiver<Result<(u8, Vec<u8>), DropWireError>>>,
}

impl ParallelStreams {
    pub async fn connect(
        addr: SocketAddr,
        _channel: &ChannelId,
        role: Role,
        auth_token: &[u8; 16],
        num_streams: u8,
    ) -> Result<Self, DropWireError> {
        let mut streams = Vec::new();
        let (tx, rx) = tokio::sync::mpsc::channel(100);

        for i in 0..num_streams {
            let mut stream = tokio::net::TcpStream::connect(addr)
                .await
                .map_err(DropWireError::Io)?;

            let tcp_channel = hex::encode(auth_token);
            let mut handshake = Vec::new();
            handshake.push(tcp_channel.len() as u8);
            handshake.extend_from_slice(tcp_channel.as_bytes());
            handshake.push(if role == Role::Sender { 0x01 } else { 0x02 });

            use tokio::io::AsyncWriteExt;
            stream
                .write_all(&handshake)
                .await
                .map_err(DropWireError::Io)?;

            let (read_half, write_half) = stream.into_split();
            streams.push(FramedStream { write: write_half });

            let tx_clone = tx.clone();
            tokio::spawn(async move {
                let mut reader = read_half;
                loop {
                    match crate::framing::read_frame(&mut reader).await {
                        Ok(bytes) => {
                            if tx_clone.send(Ok((i, bytes))).await.is_err() {
                                break;
                            }
                        }
                        Err(e) => {
                            let _ = tx_clone.send(Err(e)).await;
                            break;
                        }
                    }
                }
            });
        }

        Ok(Self {
            streams,
            rx: Some(rx),
        })
    }

    pub async fn send_raw(&mut self, stream_idx: usize, bytes: &[u8]) -> Result<(), DropWireError> {
        if stream_idx >= self.streams.len() {
            return Err(DropWireError::Network("Invalid stream index".into()));
        }
        crate::framing::write_frame(&mut self.streams[stream_idx].write, bytes).await
    }

    pub async fn send_chunk(&mut self, chunk: ChunkFrame) -> Result<(), DropWireError> {
        if self.streams.is_empty() {
            return Err(DropWireError::Network("No streams".into()));
        }
        let idx = (chunk.chunk_index % (self.streams.len() as u64)) as usize;
        let bytes = chunk.to_bytes();
        self.send_raw(idx, &bytes).await
    }

    pub async fn recv_raw(&mut self) -> Result<(u8, Vec<u8>), DropWireError> {
        if let Some(rx) = &mut self.rx {
            match rx.recv().await {
                Some(Ok(res)) => Ok(res),
                Some(Err(e)) => Err(e),
                None => Err(DropWireError::Network("All streams closed".into())),
            }
        } else {
            Err(DropWireError::Network("No receiver".into()))
        }
    }

    pub async fn recv_chunk(&mut self) -> Result<(u8, ChunkFrame), DropWireError> {
        let (idx, bytes) = self.recv_raw().await?;
        let chunk = ChunkFrame::from_bytes(&bytes)?;
        Ok((idx, chunk))
    }

    pub async fn close(mut self) -> Result<(), DropWireError> {
        for stream in &mut self.streams {
            let _ = tokio::io::AsyncWriteExt::shutdown(&mut stream.write).await;
        }
        if let Some(mut rx) = self.rx.take() {
            while let Some(_) = rx.recv().await {}
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use dashmap::DashMap;
    use std::sync::Arc;
    use tokio::net::TcpListener;

    async fn mock_relay_server() -> SocketAddr {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();

        let pending: Arc<DashMap<String, Vec<tokio::net::TcpStream>>> =
            Arc::new(DashMap::new());

        tokio::spawn(async move {
            while let Ok((mut stream, _)) = listener.accept().await {
                let pending = pending.clone();
                tokio::spawn(async move {
                    use tokio::io::AsyncReadExt;
                    let mut chan_len = [0u8; 1];
                    if stream.read_exact(&mut chan_len).await.is_err() {
                        return;
                    }
                    let mut chan_bytes = vec![0u8; chan_len[0] as usize];
                    if stream.read_exact(&mut chan_bytes).await.is_err() {
                        return;
                    }
                    let chan_str = String::from_utf8(chan_bytes).unwrap();
                    let mut role = [0u8; 1];
                    if stream.read_exact(&mut role).await.is_err() {
                        return;
                    }

                    let opposite_role = if role[0] == 0x01 { 0x02 } else { 0x01 };
                    let search_key = format!("{}_{}", chan_str, opposite_role);
                    let insert_key = format!("{}_{}", chan_str, role[0]);

                    let mut matched = None;
                    if let Some(mut peers) = pending.get_mut(&search_key) {
                        if !peers.is_empty() {
                            let peer = peers.remove(0);
                            matched = Some(peer);
                        }
                    }

                    if let Some(mut peer) = matched {
                        let _ = tokio::io::copy_bidirectional(&mut stream, &mut peer).await;
                    } else {
                        pending.entry(insert_key).or_default().push(stream);
                    }
                });
            }
        });

        addr
    }

    #[tokio::test]
    async fn test_handshake_success() {
        let addr = mock_relay_server().await;
        let chan = ChannelId("chan1".into());
        let auth = [42u8; 16];

        let sender = ParallelStreams::connect(addr, &chan, Role::Sender, &auth, 4);
        let receiver = ParallelStreams::connect(addr, &chan, Role::Receiver, &auth, 4);

        let (s, r) = tokio::join!(sender, receiver);
        assert!(s.is_ok());
        assert!(r.is_ok());

        let s = s.unwrap();
        let r = r.unwrap();
        assert_eq!(s.streams.len(), 4);
        assert_eq!(r.streams.len(), 4);
    }

    #[tokio::test]
    async fn test_round_robin_chunks() {
        let addr = mock_relay_server().await;
        let chan = ChannelId("chan2".into());
        let auth = [42u8; 16];

        let mut s = ParallelStreams::connect(addr, &chan, Role::Sender, &auth, 4)
            .await
            .unwrap();
        let mut r = ParallelStreams::connect(addr, &chan, Role::Receiver, &auth, 4)
            .await
            .unwrap();

        for i in 0..100 {
            s.send_chunk(ChunkFrame {
                chunk_index: i,
                is_compressed: false,
                data: vec![i as u8; 10],
            })
            .await
            .unwrap();
        }

        let mut received = 0;
        while received < 100 {
            let (idx, chunk) = r.recv_chunk().await.unwrap();
            assert_eq!(idx as u64, chunk.chunk_index % 4);
            assert_eq!(chunk.data.len(), 10);
            assert_eq!(chunk.data[0], chunk.chunk_index as u8);
            received += 1;
        }
    }

    #[tokio::test]
    async fn test_stream_dies() {
        let addr = mock_relay_server().await;
        let chan = ChannelId("chan3".into());
        let auth = [42u8; 16];

        let s = ParallelStreams::connect(addr, &chan, Role::Sender, &auth, 4)
            .await
            .unwrap();
        let mut r = ParallelStreams::connect(addr, &chan, Role::Receiver, &auth, 4)
            .await
            .unwrap();

        drop(s);

        let res = r.recv_chunk().await;
        assert!(res.is_err());
    }

    #[tokio::test]
    async fn test_auth_mismatch() {
        let addr = mock_relay_server().await;
        let chan = ChannelId("chan4".into());

        let _s = ParallelStreams::connect(addr, &chan, Role::Sender, &[1u8; 16], 4)
            .await
            .unwrap();
        let mut r = ParallelStreams::connect(addr, &chan, Role::Receiver, &[2u8; 16], 4)
            .await
            .unwrap();

        tokio::time::sleep(std::time::Duration::from_millis(50)).await;

        let res = tokio::time::timeout(std::time::Duration::from_millis(200), r.recv_chunk()).await;
        assert!(res.is_err());
    }
}
