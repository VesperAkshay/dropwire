use crate::crypto::kdf::derive_auth_token;
use crate::crypto::pake::PakeState;
use crate::error::DropWireError;
use crate::types::ChannelId;
use base64::{engine::general_purpose::STANDARD as BASE64, Engine as _};
use futures::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use std::time::Duration;
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};

#[derive(Clone, Copy, PartialEq)]
pub enum Role {
    Sender,
    Receiver,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
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
    DirectCandidate { ip: String, port: u16 },
}

pub struct SignalingResult {
    pub shared_key: [u8; 32],
    pub auth_token: [u8; 16],
    pub peer_ready: bool,
}

pub struct SignalingClient;

impl SignalingClient {
    pub async fn connect(
        relay_ws_url: &str,
        channel: &ChannelId,
        role: Role,
        password: &str,
    ) -> Result<SignalingResult, DropWireError> {
        tokio::time::timeout(
            Duration::from_secs(60),
            Self::connect_inner(relay_ws_url, channel, role, password),
        )
        .await
        .map_err(|_| DropWireError::Network("Timeout".into()))?
    }

    async fn connect_inner(
        relay_ws_url: &str,
        channel: &ChannelId,
        role: Role,
        password: &str,
    ) -> Result<SignalingResult, DropWireError> {
        let (ws_stream, _) = connect_async(relay_ws_url)
            .await
            .map_err(|_| DropWireError::Network("WS connect failed".into()))?;
        let (mut write, mut read) = ws_stream.split();

        // 1. Send Register
        let role_str = if role == Role::Sender {
            "sender"
        } else {
            "receiver"
        };
        let reg = WsMsg::Register {
            role: role_str.to_string(),
            channel: channel.0.clone(),
        };
        write
            .send(Message::Text(serde_json::to_string(&reg).unwrap()))
            .await
            .map_err(|_| DropWireError::Network("WS send failed".into()))?;

        // 2. Wait for Registered
        loop {
            let msg = read
                .next()
                .await
                .ok_or(DropWireError::Network("WS closed".into()))?
                .map_err(|_| DropWireError::Network("WS error".into()))?;
            if let Message::Text(text) = msg {
                let ws_msg: WsMsg = serde_json::from_str(&text)
                    .map_err(|_| DropWireError::Protocol("Bad JSON".into()))?;
                match ws_msg {
                    WsMsg::Registered => break,
                    WsMsg::Error { message, .. } => return Err(DropWireError::Protocol(message)),
                    _ => continue,
                }
            }
        }

        // 3. Init PAKE
        let is_alice = role == Role::Sender; // Sender acts as Alice
        let (pake, outbound) = PakeState::new(password, &channel.0, is_alice);

        let scalar_b64 = BASE64.encode(outbound);
        let step_msg = if role == Role::Sender {
            WsMsg::PakeStep1 {
                scalar_mul: scalar_b64,
            }
        } else {
            WsMsg::PakeStep2 {
                scalar_mul: scalar_b64,
            }
        };
        write
            .send(Message::Text(serde_json::to_string(&step_msg).unwrap()))
            .await
            .map_err(|_| DropWireError::Network("WS send failed".into()))?;

        // 4. Wait for peer's PAKE step
        let peer_b64 = loop {
            let msg = read
                .next()
                .await
                .ok_or(DropWireError::Network("WS closed".into()))?
                .map_err(|_| DropWireError::Network("WS error".into()))?;
            if let Message::Text(text) = msg {
                let ws_msg: WsMsg = serde_json::from_str(&text)
                    .map_err(|_| DropWireError::Protocol("Bad JSON".into()))?;
                match ws_msg {
                    WsMsg::PakeStep1 { scalar_mul } if role == Role::Receiver => break scalar_mul,
                    WsMsg::PakeStep2 { scalar_mul } if role == Role::Sender => break scalar_mul,
                    WsMsg::Error { message, .. } => return Err(DropWireError::Protocol(message)),
                    _ => continue,
                }
            }
        };

        let peer_bytes = BASE64
            .decode(&peer_b64)
            .map_err(|_| DropWireError::Protocol("Bad Base64".into()))?;
        if peer_bytes.len() != 32 {
            return Err(DropWireError::Crypto("Invalid PAKE point".into()));
        }
        let mut peer_arr = [0u8; 32];
        peer_arr.copy_from_slice(&peer_bytes);

        // 5. Finish PAKE and derive token
        let shared_secret = pake.finish(&peer_arr)?;
        let auth_token = derive_auth_token(&shared_secret.0);
        let auth_hex = hex::encode(auth_token);

        // 6. Send AuthProof
        let auth_msg = WsMsg::AuthProof { token: auth_hex };
        write
            .send(Message::Text(serde_json::to_string(&auth_msg).unwrap()))
            .await
            .map_err(|_| DropWireError::Network("WS send failed".into()))?;

        // 7. Wait for PeerReady
        loop {
            let msg = read
                .next()
                .await
                .ok_or(DropWireError::Network("WS closed".into()))?
                .map_err(|_| DropWireError::Network("WS error".into()))?;
            if let Message::Text(text) = msg {
                let ws_msg: WsMsg = serde_json::from_str(&text)
                    .map_err(|_| DropWireError::Protocol("Bad JSON".into()))?;
                match ws_msg {
                    WsMsg::PeerReady => break,
                    WsMsg::AuthFail => {
                        return Err(DropWireError::Crypto("auth proof mismatch".into()))
                    }
                    WsMsg::Error { message, .. } => return Err(DropWireError::Protocol(message)),
                    _ => continue,
                }
            }
        }

        Ok(SignalingResult {
            shared_key: shared_secret.0,
            auth_token,
            peer_ready: true,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use dashmap::DashMap;
    use std::sync::Arc;
    use tokio::net::TcpListener;
    use tokio_tungstenite::accept_async;

    struct MockSlot {
        sender: Option<futures::channel::mpsc::UnboundedSender<Message>>,
        receiver: Option<futures::channel::mpsc::UnboundedSender<Message>>,
        sender_token: Option<String>,
        receiver_token: Option<String>,
    }

    async fn mock_relay_server() -> String {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let port = listener.local_addr().unwrap().port();
        let url = format!("ws://127.0.0.1:{}", port);

        let slots: Arc<DashMap<String, MockSlot>> = Arc::new(DashMap::new());

        tokio::spawn(async move {
            while let Ok((stream, _)) = listener.accept().await {
                let ws = accept_async(stream).await.unwrap();
                let (mut ws_tx, mut ws_rx) = ws.split();
                let (tx, mut rx) = futures::channel::mpsc::unbounded();

                let slots = slots.clone();
                tokio::spawn(async move {
                    while let Some(msg) = rx.next().await {
                        let _ = ws_tx.send(msg).await;
                    }
                });

                tokio::spawn(async move {
                    let mut my_role = "".to_string();
                    let mut my_chan = "".to_string();
                    while let Some(Ok(msg)) = ws_rx.next().await {
                        if let Message::Text(txt) = msg {
                            let w: WsMsg = serde_json::from_str(&txt).unwrap();
                            match w {
                                WsMsg::Register { role, channel } => {
                                    my_role = role.clone();
                                    my_chan = channel.clone();
                                    let mut entry = slots.entry(channel).or_insert(MockSlot {
                                        sender: None,
                                        receiver: None,
                                        sender_token: None,
                                        receiver_token: None,
                                    });
                                    if role == "sender" {
                                        entry.sender = Some(tx.clone());
                                    } else {
                                        entry.receiver = Some(tx.clone());
                                    }
                                    let _ = tx.unbounded_send(Message::Text(
                                        serde_json::to_string(&WsMsg::Registered).unwrap(),
                                    ));
                                }
                                WsMsg::PakeStep1 { .. } => {
                                    if let Some(entry) = slots.get(&my_chan) {
                                        if let Some(rx_tx) = &entry.receiver {
                                            let _ = rx_tx.unbounded_send(Message::Text(txt));
                                        }
                                    }
                                }
                                WsMsg::PakeStep2 { .. } => {
                                    if let Some(entry) = slots.get(&my_chan) {
                                        if let Some(tx_tx) = &entry.sender {
                                            let _ = tx_tx.unbounded_send(Message::Text(txt));
                                        }
                                    }
                                }
                                WsMsg::AuthProof { token } => {
                                    let mut ready = false;
                                    let mut fail = false;
                                    if let Some(mut entry) = slots.get_mut(&my_chan) {
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
                                        let entry = slots.get(&my_chan).unwrap();
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
                                    } else if fail {
                                        let entry = slots.get(&my_chan).unwrap();
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
                                }
                                _ => {}
                            }
                        }
                    }
                });
            }
        });

        url
    }

    #[tokio::test]
    async fn test_signaling_success() {
        let url = mock_relay_server().await;

        let chan = ChannelId("test1".into());
        let sender_fut = SignalingClient::connect(&url, &chan, Role::Sender, "7-guitar-abandon");
        let receiver_fut =
            SignalingClient::connect(&url, &chan, Role::Receiver, "7-guitar-abandon");

        let (s_res, r_res) = tokio::join!(sender_fut, receiver_fut);

        let s = s_res.unwrap();
        let r = r_res.unwrap();

        assert_eq!(s.shared_key, r.shared_key);
        assert_eq!(s.auth_token, r.auth_token);
        assert!(s.peer_ready);
        assert!(r.peer_ready);
    }

    #[tokio::test]
    async fn test_wrong_password() {
        let url = mock_relay_server().await;

        let chan = ChannelId("test2".into());
        let sender_fut = SignalingClient::connect(&url, &chan, Role::Sender, "7-guitar-abandon");
        let receiver_fut = SignalingClient::connect(&url, &chan, Role::Receiver, "7-wrong-pass");

        let (s_res, r_res) = tokio::join!(sender_fut, receiver_fut);

        assert!(matches!(s_res, Err(DropWireError::Crypto(_))));
        assert!(matches!(r_res, Err(DropWireError::Crypto(_))));
    }

    #[tokio::test]
    async fn test_timeout() {
        // Just start one client, it will time out waiting for peer.
        let url = mock_relay_server().await;

        let chan = ChannelId("test3".into());
        let sender_fut = tokio::time::timeout(
            Duration::from_millis(500),
            SignalingClient::connect(&url, &chan, Role::Sender, "7-guitar-abandon"),
        );

        assert!(sender_fut.await.is_err()); // tokio timeout err
    }

    #[tokio::test]
    async fn test_server_error() {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let port = listener.local_addr().unwrap().port();
        let url = format!("ws://127.0.0.1:{}", port);

        tokio::spawn(async move {
            let (stream, _) = listener.accept().await.unwrap();
            let mut ws = accept_async(stream).await.unwrap();

            // Wait for Register
            let _ = ws.next().await;

            // Send Error
            let err = WsMsg::Error {
                code: "TEST".into(),
                message: "Server died".into(),
            };
            let _ = ws
                .send(Message::Text(serde_json::to_string(&err).unwrap()))
                .await;
        });

        let chan = ChannelId("test4".into());
        let res = SignalingClient::connect(&url, &chan, Role::Sender, "7-guitar-abandon").await;
        assert!(matches!(res, Err(DropWireError::Protocol(_))));
    }
}
