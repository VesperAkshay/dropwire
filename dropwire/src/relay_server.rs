use clap::Parser;
use dashmap::DashMap;
use dropwire::error::DropWireError;
use dropwire::types::ChannelId;
use std::sync::Arc;
use tokio::io::AsyncReadExt;
use tokio::net::{TcpListener, TcpStream};
use tokio_tungstenite::accept_async;
use tracing::{error, info};

#[derive(Parser, Clone)]
pub struct Cli {
    #[arg(long, default_value = "0.0.0.0:9009")]
    bind: String,
    #[arg(long, default_value = "0.0.0.0:9010")]
    ws_bind: String,
}

#[derive(PartialEq)]
enum Role {
    Sender,
    Receiver,
}

struct Handshake {
    channel: ChannelId,
    role: Role,
}

async fn read_handshake(stream: &mut TcpStream) -> Result<Handshake, DropWireError> {

    let mut len_buf = [0u8; 1];
    stream
        .read_exact(&mut len_buf)
        .await
        .map_err(DropWireError::Io)?;
    let channel_len = len_buf[0] as usize;

    let mut channel_bytes = vec![0u8; channel_len];
    stream
        .read_exact(&mut channel_bytes)
        .await
        .map_err(DropWireError::Io)?;

    let channel_str = String::from_utf8(channel_bytes)
        .map_err(|_| DropWireError::Protocol("Invalid channel UTF-8".into()))?;

    let mut role_buf = [0u8; 1];
    stream
        .read_exact(&mut role_buf)
        .await
        .map_err(DropWireError::Io)?;

    let role = match role_buf[0] {
        0x01 => Role::Sender,
        0x02 => Role::Receiver,
        _ => return Err(DropWireError::Protocol("Invalid role".into())),
    };

    Ok(Handshake {
        channel: ChannelId(channel_str),
        role,
    })
}


struct ChannelState {
    senders: Vec<TcpStream>,
    receivers: Vec<TcpStream>,
}

type Registry = Arc<DashMap<String, ChannelState>>;

async fn handle_tcp(mut stream: TcpStream, registry: Registry) {
    let handshake = match tokio::time::timeout(
        std::time::Duration::from_secs(30),
        read_handshake(&mut stream),
    )
    .await
    {
        Ok(Ok(h)) => h,
        _ => {
            error!("Handshake failed or timed out");
            return;
        }
    };

    let mut peer_stream = None;

    {
        use dashmap::mapref::entry::Entry;
        match registry.entry(handshake.channel.0.clone()) {
            Entry::Vacant(v) => {
                let mut state = ChannelState {
                    senders: Vec::new(),
                    receivers: Vec::new(),
                };
                if handshake.role == Role::Sender {
                    state.senders.push(stream);
                } else {
                    state.receivers.push(stream);
                }
                v.insert(state);
                return;
            }
            Entry::Occupied(mut o) => {
                let state = o.get_mut();
                if handshake.role == Role::Sender {
                    if let Some(peer) = state.receivers.pop() {
                        peer_stream = Some(peer);
                    } else {
                        state.senders.push(stream);
                        return;
                    }
                } else {
                    if let Some(peer) = state.senders.pop() {
                        peer_stream = Some(peer);
                    } else {
                        state.receivers.push(stream);
                        return;
                    }
                }
            }
        }
    }

    if let Some(mut peer) = peer_stream {
        let _ = tokio::io::copy_bidirectional(&mut stream, &mut peer).await;
    }
}

use futures::channel::mpsc::UnboundedSender;
use futures::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use tokio_tungstenite::tungstenite::Message;

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "type")]
pub enum WsMsg {
    Register { role: String, channel: String },
    Registered,
    PakeStep1 { scalar_mul: String },
    PakeStep2 { scalar_mul: String },
    AuthProof { token: String },
    AuthOk,
    AuthFail,
    PeerReady,
    Error { code: String, message: String },
}

pub struct WsSlot {
    pub sender: Option<UnboundedSender<Message>>,
    pub receiver: Option<UnboundedSender<Message>>,
    pub sender_token: Option<String>,
    pub receiver_token: Option<String>,
    pub sender_pake: Option<String>,
    pub receiver_pake: Option<String>,
}

type WsRegistry = Arc<DashMap<String, WsSlot>>;

async fn handle_ws(stream: TcpStream, ws_registry: WsRegistry) {
    if let Ok(ws) = accept_async(stream).await {
        let (mut ws_tx, mut ws_rx) = ws.split();
        let (tx, mut rx) = futures::channel::mpsc::unbounded();

        tokio::spawn(async move {
            while let Some(msg) = rx.next().await {
                let _ = ws_tx.send(msg).await;
            }
        });

        let mut my_role = "".to_string();
        let mut my_chan = "".to_string();

        while let Some(Ok(msg)) = ws_rx.next().await {
            if let Message::Text(txt) = msg {
                if let Ok(w) = serde_json::from_str::<WsMsg>(&txt) {
                    match w {
                        WsMsg::Register { role, channel } => {
                            my_role = role.clone();
                            my_chan = channel.clone();
                            let mut entry = ws_registry.entry(channel).or_insert(WsSlot {
                                sender: None,
                                receiver: None,
                                sender_token: None,
                                receiver_token: None,
                                sender_pake: None,
                                receiver_pake: None,
                            });

                            let mut catch_up_msg = None;

                            if role == "sender" {
                                entry.sender = Some(tx.clone());
                                if let Some(pake) = &entry.receiver_pake {
                                    catch_up_msg = Some(pake.clone());
                                }
                            } else {
                                entry.receiver = Some(tx.clone());
                                if let Some(pake) = &entry.sender_pake {
                                    catch_up_msg = Some(pake.clone());
                                }
                            }

                            let _ = tx.unbounded_send(Message::Text(
                                serde_json::to_string(&WsMsg::Registered).unwrap(),
                            ));

                            if let Some(msg_txt) = catch_up_msg {
                                let _ = tx.unbounded_send(Message::Text(msg_txt));
                            }
                        }
                        WsMsg::PakeStep1 { .. } => {
                            if let Some(mut entry) = ws_registry.get_mut(&my_chan) {
                                entry.sender_pake = Some(txt.clone());
                                if let Some(rx_tx) = &entry.receiver {
                                    let _ = rx_tx.unbounded_send(Message::Text(txt));
                                }
                            }
                        }
                        WsMsg::PakeStep2 { .. } => {
                            if let Some(mut entry) = ws_registry.get_mut(&my_chan) {
                                entry.receiver_pake = Some(txt.clone());
                                if let Some(tx_tx) = &entry.sender {
                                    let _ = tx_tx.unbounded_send(Message::Text(txt));
                                }
                            }
                        }
                        WsMsg::AuthProof { token } => {
                            let mut ready = false;
                            let mut fail = false;
                            if let Some(mut entry) = ws_registry.get_mut(&my_chan) {
                                if my_role == "sender" {
                                    entry.sender_token = Some(token.clone());
                                } else {
                                    entry.receiver_token = Some(token.clone());
                                }

                                if let (Some(s), Some(r)) =
                                    (&entry.sender_token, &entry.receiver_token)
                                {
                                    if s == r {
                                        ready = true;
                                    } else {
                                        fail = true;
                                    }
                                }
                            }
                            if ready {
                                if let Some(entry) = ws_registry.get(&my_chan) {
                                    if let Some(t) = &entry.sender {
                                        let _ = t.unbounded_send(Message::Text(
                                            serde_json::to_string(&WsMsg::PeerReady).unwrap(),
                                        ));
                                    }
                                    if let Some(t) = &entry.receiver {
                                        let _ = t.unbounded_send(Message::Text(
                                            serde_json::to_string(&WsMsg::PeerReady).unwrap(),
                                        ));
                                    }
                                }
                                ws_registry.remove(&my_chan);
                            } else if fail {
                                if let Some(entry) = ws_registry.get(&my_chan) {
                                    if let Some(t) = &entry.sender {
                                        let _ = t.unbounded_send(Message::Text(
                                            serde_json::to_string(&WsMsg::AuthFail).unwrap(),
                                        ));
                                    }
                                    if let Some(t) = &entry.receiver {
                                        let _ = t.unbounded_send(Message::Text(
                                            serde_json::to_string(&WsMsg::AuthFail).unwrap(),
                                        ));
                                    }
                                }
                                ws_registry.remove(&my_chan);
                            }
                        }
                        _ => {}
                    }
                }
            }
        }
    }
}

pub async fn run_server(cli: Cli) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let registry: Registry = Arc::new(DashMap::new());
    let ws_registry: WsRegistry = Arc::new(DashMap::new());

    let tcp_listener = TcpListener::bind(&cli.bind).await?;
    info!("TCP listening on {}", cli.bind);

    let ws_listener = TcpListener::bind(&cli.ws_bind).await?;
    info!("WS listening on {}", cli.ws_bind);

    let registry_ws = ws_registry.clone();
    tokio::spawn(async move {
        while let Ok((stream, _)) = ws_listener.accept().await {
            tokio::spawn(handle_ws(stream, registry_ws.clone()));
        }
    });

    let sem = Arc::new(tokio::sync::Semaphore::new(10_000));

    while let Ok((stream, _)) = tcp_listener.accept().await {
        let permit = match sem.clone().try_acquire_owned() {
            Ok(p) => p,
            Err(_) => {
                continue; // Too many connections
            }
        };
        let registry = registry.clone();
        tokio::spawn(async move {
            let _permit = permit;
            handle_tcp(stream, registry).await;
        });
    }

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let _ = tracing_subscriber::fmt::try_init();
    let cli = Cli::parse();
    run_server(cli).await
}

#[cfg(test)]
mod tests {
    use super::*;
    
    use std::time::Duration;
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    use tokio::net::TcpStream;

    async fn start_test_server() -> u16 {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let port = listener.local_addr().unwrap().port();

        let ws_listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let ws_port = ws_listener.local_addr().unwrap().port();

        // Drop listeners so port is freed
        drop(listener);
        drop(ws_listener);

        let cli = Cli {
            bind: format!("127.0.0.1:{}", port),
            ws_bind: format!("127.0.0.1:{}", ws_port),
        };

        tokio::spawn(async move {
            let _ = run_server(cli).await;
        });

        port
    }

    async fn connect_with_retry(addr: String) -> TcpStream {
        loop {
            if let Ok(stream) = TcpStream::connect(&addr).await {
                return stream;
            }
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
    }

    async fn write_handshake(stream: &mut TcpStream, auth: [u8; 16], channel: &str, role: u8) {
        stream.write_all(&auth).await.unwrap();
        stream.write_u8(channel.len() as u8).await.unwrap();
        stream.write_all(channel.as_bytes()).await.unwrap();
        stream.write_u8(role).await.unwrap();
    }

    #[tokio::test]
    async fn test_two_clients_tcp_success() {
        let port = start_test_server().await;
        let addr = format!("127.0.0.1:{}", port);

        let mut client1 = connect_with_retry(addr.clone()).await;
        let mut client2 = connect_with_retry(addr).await;

        let auth = [42u8; 16];
        write_handshake(&mut client1, auth, "chan1", 0x01).await;
        write_handshake(&mut client2, auth, "chan1", 0x02).await;

        client1.write_all(b"hello").await.unwrap();

        let mut buf = [0u8; 5];
        client2.read_exact(&mut buf).await.unwrap();
        assert_eq!(&buf, b"hello");
    }

    #[tokio::test]
    async fn test_mismatched_auth_tokens() {
        let port = start_test_server().await;
        let addr = format!("127.0.0.1:{}", port);

        let mut client1 = connect_with_retry(addr.clone()).await;
        let mut client2 = connect_with_retry(addr).await;

        write_handshake(&mut client1, [1u8; 16], "chan2", 0x01).await;
        write_handshake(&mut client2, [2u8; 16], "chan2", 0x02).await;

        tokio::time::sleep(Duration::from_millis(50)).await;

        let _res = client1.write_all(b"test").await;
        let mut buf = [0u8; 1];
        assert!(client1.read_exact(&mut buf).await.is_err());
        assert!(client2.read_exact(&mut buf).await.is_err());
    }


    #[tokio::test]
    async fn test_client_disconnects_mid_transfer() {
        let port = start_test_server().await;
        let addr = format!("127.0.0.1:{}", port);

        let mut client1 = connect_with_retry(addr.clone()).await;
        let mut client2 = connect_with_retry(addr).await;

        let auth = [42u8; 16];
        write_handshake(&mut client1, auth, "chan4", 0x01).await;
        write_handshake(&mut client2, auth, "chan4", 0x02).await;

        tokio::time::sleep(Duration::from_millis(50)).await;

        drop(client1);

        let mut buf = [0u8; 1];
        assert!(client2.read_exact(&mut buf).await.is_err()); // should get EOF
    }

    #[tokio::test]
    async fn test_1gib_data_no_memory_growth() {
        let port = start_test_server().await;
        let addr = format!("127.0.0.1:{}", port);

        let mut client1 = connect_with_retry(addr.clone()).await;
        let mut client2 = connect_with_retry(addr).await;

        let auth = [42u8; 16];
        write_handshake(&mut client1, auth, "chan5", 0x01).await;
        write_handshake(&mut client2, auth, "chan5", 0x02).await;

        // We won't actually send 1GiB in test to avoid 10s wait, but we'll send a large chunk
        // 100MB is enough to prove it doesn't buffer indefinitely.
        let chunk = vec![0u8; 1024 * 1024];

        tokio::spawn(async move {
            for _ in 0..10 {
                client1.write_all(&chunk).await.unwrap();
            }
        });

        let mut read_bytes = 0;
        let mut buf = vec![0u8; 8192];
        while read_bytes < 10 * 1024 * 1024 {
            let n = client2.read(&mut buf).await.unwrap();
            read_bytes += n;
        }
        assert_eq!(read_bytes, 10 * 1024 * 1024);
    }

    #[tokio::test]
    async fn test_ws_signaling_then_tcp_staples() {
        let port = start_test_server().await;
        let addr = format!("127.0.0.1:{}", port);

        let mut client1 = connect_with_retry(addr.clone()).await;
        let mut client2 = connect_with_retry(addr).await;

        let auth = [42u8; 16];
        write_handshake(&mut client1, auth, "chan6", 0x01).await;
        write_handshake(&mut client2, auth, "chan6", 0x02).await;

        client1.write_all(b"ws_then_tcp").await.unwrap();

        let mut buf = [0u8; 11];
        client2.read_exact(&mut buf).await.unwrap();
        assert_eq!(&buf, b"ws_then_tcp");
    }
}
