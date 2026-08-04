use std::path::PathBuf;
use tokio::sync::mpsc;
use rand::Rng;

use dropwire::crypto::kdf::{derive_control_key, derive_stream_key};
use dropwire::net::parallel::ParallelStreams;
use dropwire::net::signaling::{Role, SignalingClient};
use dropwire::transfer::engine::TransferEngine;
use dropwire::types::ChannelId;

pub enum EngineEvent {
    InitSend(String), // The code phrase
    InitReceive(String),
    Status(String),
    Progress { current: u64, total: u64 },
    Done,
    Error(String),
}

fn generate_code() -> String {
    let words = dropwire::wordlist::BIP39_WORDS;
    let mut rng = rand::thread_rng();
    let num = rng.gen_range(1..=999);
    let word1 = words[rng.gen_range(0..words.len())];
    let word2 = words[rng.gen_range(0..words.len())];
    format!("{}-{}-{}", num, word1, word2)
}

// Use the exact same default relay as the CLI
const RELAY_URL: &str = dropwire::cli::config::DEFAULT_RELAY;

pub async fn start_send(paths: Vec<PathBuf>, tx: mpsc::UnboundedSender<crate::AppEvent>) {
    let code_phrase = generate_code();
    let _ = tx.send(crate::AppEvent::Engine(EngineEvent::InitSend(code_phrase.clone())));
    let channel_id = ChannelId::derive(&code_phrase);
    let mut retry_count = 0;

    loop {
        let _ = tx.send(crate::AppEvent::Engine(EngineEvent::Status(
            if retry_count == 0 { "Waiting for peer to connect...".to_string() } 
            else { format!("⚠ Connection lost. Reconnecting... (Attempt {})", retry_count) }
        )));

        let sig_result = match SignalingClient::connect(RELAY_URL, &channel_id, Role::Sender, &code_phrase).await {
            Ok(res) => res,
            Err(_) => {
                retry_count += 1;
                tokio::time::sleep(std::time::Duration::from_secs(3)).await;
                continue;
            }
        };

        let _ = tx.send(crate::AppEvent::Engine(EngineEvent::Status("Peer connected. Starting transfer...".to_string())));

        let config = dropwire::cli::config::DropwireConfig::load();
        let tcp_relay = RELAY_URL.replace("ws://", "").replace("9010", "9009");
        
        let parallel_res = if !config.get_no_lan() {
            if let Ok(listener) = dropwire::net::parallel::ParallelListener::bind().await {
                let port = listener.port;
                let chan = channel_id.clone();
                let _ = tx.send(crate::AppEvent::Engine(EngineEvent::Status(format!("⚡ LAN Discovery: Listening on port {}...", port))));
                tokio::spawn(async move {
                    let _ = dropwire::net::discovery::DiscoveryService::announce(&chan, std::time::Duration::from_secs(30), port).await;
                });
                match tokio::time::timeout(std::time::Duration::from_secs(15), listener.accept_all(&sig_result.auth_token, config.get_parallel_streams())).await {
                    Ok(Ok(p)) => {
                        let _ = tx.send(crate::AppEvent::Engine(EngineEvent::Status("✓ Connected via LAN (Direct P2P) — Maximum speed!".to_string())));
                        Ok(p)
                    }
                    _ => {
                        let _ = tx.send(crate::AppEvent::Engine(EngineEvent::Status("⚠ LAN peer not found, falling back to Relay...".to_string())));
                        ParallelStreams::connect(tcp_relay.clone(), &channel_id, Role::Sender, &sig_result.auth_token, config.get_parallel_streams()).await
                    }
                }
            } else {
                ParallelStreams::connect(tcp_relay.clone(), &channel_id, Role::Sender, &sig_result.auth_token, config.get_parallel_streams()).await
            }
        } else {
            ParallelStreams::connect(tcp_relay.clone(), &channel_id, Role::Sender, &sig_result.auth_token, config.get_parallel_streams()).await
        };

        let parallel = match parallel_res {
            Ok(res) => res,
            Err(_) => {
                retry_count += 1;
                tokio::time::sleep(std::time::Duration::from_secs(3)).await;
                continue;
            }
        };

        let engine = TransferEngine::new(
            derive_stream_key(&sig_result.shared_key, 0),
            derive_control_key(&sig_result.shared_key),
        );

        let tx_clone = tx.clone();
        let res = engine.send(&paths, parallel, config.get_chunk_size_kb(), move |current, total, _| {
            let _ = tx_clone.send(crate::AppEvent::Engine(EngineEvent::Progress { current, total }));
        }).await;

        if let Err(_) = res {
            retry_count += 1;
            tokio::time::sleep(std::time::Duration::from_secs(3)).await;
            continue;
        } else {
            let _ = tx.send(crate::AppEvent::Engine(EngineEvent::Status("Transfer Complete!".to_string())));
            let _ = tx.send(crate::AppEvent::Engine(EngineEvent::Done));
            break;
        }
    }
}

pub async fn start_receive(code_phrase: String, mut out_dir: PathBuf, tx: mpsc::UnboundedSender<crate::AppEvent>) {
    let config = dropwire::cli::config::DropwireConfig::load();
    if let Some(custom_dir) = config.get_download_dir() {
        out_dir = custom_dir;
    } else if let Some(mut downloads) = dirs::download_dir() {
        downloads.push("Dropwire");
        let _ = std::fs::create_dir_all(&downloads);
        out_dir = downloads;
    }

    let _ = tx.send(crate::AppEvent::Engine(EngineEvent::InitReceive(code_phrase.clone())));
    let channel_id = ChannelId::derive(&code_phrase);
    let mut retry_count = 0;

    loop {
        let _ = tx.send(crate::AppEvent::Engine(EngineEvent::Status(
            if retry_count == 0 { "Connecting to peer...".to_string() }
            else { format!("⚠ Connection lost. Reconnecting... (Attempt {})", retry_count) }
        )));

        let sig_result = match SignalingClient::connect(RELAY_URL, &channel_id, Role::Receiver, &code_phrase).await {
            Ok(res) => res,
            Err(_) => {
                retry_count += 1;
                tokio::time::sleep(std::time::Duration::from_secs(3)).await;
                continue;
            }
        };

        let _ = tx.send(crate::AppEvent::Engine(EngineEvent::Status("Peer connected. Receiving files...".to_string())));

        let tcp_relay = RELAY_URL.replace("ws://", "").replace("9010", "9009");
        
        let parallel_res = if !config.get_no_lan() {
            let _ = tx.send(crate::AppEvent::Engine(EngineEvent::Status("⚡ LAN Discovery: Searching for peer...".to_string())));
            match dropwire::net::discovery::DiscoveryService::find_peer(&channel_id, std::time::Duration::from_secs(15)).await {
                Ok(peer_addr) => {
                    let _ = tx.send(crate::AppEvent::Engine(EngineEvent::Status(format!("✓ Found peer at {} — Direct P2P!", peer_addr))));
                    ParallelStreams::connect(peer_addr.to_string(), &channel_id, Role::Receiver, &sig_result.auth_token, config.get_parallel_streams()).await
                }
                Err(_) => {
                    let _ = tx.send(crate::AppEvent::Engine(EngineEvent::Status("⚠ LAN peer not found, falling back to Relay...".to_string())));
                    ParallelStreams::connect(tcp_relay.clone(), &channel_id, Role::Receiver, &sig_result.auth_token, config.get_parallel_streams()).await
                }
            }
        } else {
            ParallelStreams::connect(tcp_relay.clone(), &channel_id, Role::Receiver, &sig_result.auth_token, config.get_parallel_streams()).await
        };

        let parallel = match parallel_res {
            Ok(res) => res,
            Err(_) => {
                retry_count += 1;
                tokio::time::sleep(std::time::Duration::from_secs(3)).await;
                continue;
            }
        };

        let engine = TransferEngine::new(
            derive_stream_key(&sig_result.shared_key, 0),
            derive_control_key(&sig_result.shared_key),
        );

        let tx_clone = tx.clone();
        let res = engine.receive(&out_dir, parallel, move |current, total, _| {
            let _ = tx_clone.send(crate::AppEvent::Engine(EngineEvent::Progress { current, total }));
        }).await;

        if let Err(_) = res {
            retry_count += 1;
            tokio::time::sleep(std::time::Duration::from_secs(3)).await;
            continue;
        } else {
            let _ = tx.send(crate::AppEvent::Engine(EngineEvent::Status("Transfer Complete!".to_string())));
            let _ = tx.send(crate::AppEvent::Engine(EngineEvent::Done));
            break;
        }
    }
}
