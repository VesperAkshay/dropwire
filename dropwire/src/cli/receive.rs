use crate::crypto::kdf::{derive_control_key, derive_stream_key};
use crate::error::DropWireError;
use crate::net::parallel::ParallelStreams;
use crate::net::signaling::{Role, SignalingClient};
use crate::transfer::engine::TransferEngine;
use crate::types::ChannelId;
use indicatif::{ProgressBar, ProgressStyle};
use std::path::PathBuf;
pub async fn run(
    code: String,
    out_dir: Option<PathBuf>,
    relay_url: String,
    no_lan: bool,
) -> Result<(), DropWireError> {
    let out_dir = out_dir.unwrap_or_else(|| {
        dirs::download_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("Dropwire")
    });

    if !out_dir.exists() {
        if let Err(e) = std::fs::create_dir_all(&out_dir) {
            eprintln!("Error: Could not create output directory: {}", e);
            std::process::exit(1);
        }
    } else if !out_dir.is_dir() {
        eprintln!("Error: Output path exists but is not a directory.");
        std::process::exit(1);
    }

    println!("Connecting...");

    let channel_id = ChannelId::derive(&code);

    let sig_result =
        SignalingClient::connect(&relay_url, &channel_id, Role::Receiver, &code).await?;

    let tcp_relay = relay_url.replace("ws://", "").replace("9010", "9009");
    let parallel = if !no_lan {
        println!("\x1b[36m⚡ LAN Discovery: Searching for peer on local network...\x1b[0m");
        match crate::net::discovery::DiscoveryService::find_peer(&channel_id, std::time::Duration::from_secs(15)).await {
            Ok(peer_addr) => {
                println!("\x1b[32m✓ Found peer at {} — Direct P2P connection!\x1b[0m", peer_addr);
                match ParallelStreams::connect(
                    peer_addr.to_string(),
                    &channel_id,
                    Role::Receiver,
                    &sig_result.auth_token,
                    4,
                ).await {
                    Ok(p) => p,
                    Err(_) => {
                        println!("\x1b[33m⚠ LAN TCP connection failed (firewall?), falling back to Relay...\x1b[0m");
                        ParallelStreams::connect(
                            tcp_relay.clone(),
                            &channel_id,
                            Role::Receiver,
                            &sig_result.auth_token,
                            4,
                        ).await?
                    }
                }
            }
            Err(_) => {
                println!("\x1b[33m⚠ LAN peer not found, falling back to Relay...\x1b[0m");
                ParallelStreams::connect(
                    tcp_relay,
                    &channel_id,
                    Role::Receiver,
                    &sig_result.auth_token,
                    4,
                ).await?
            }
        }
    } else {
        println!("\x1b[33m⚠ LAN disabled by config, using Relay...\x1b[0m");
        ParallelStreams::connect(
            tcp_relay,
            &channel_id,
            Role::Receiver,
            &sig_result.auth_token,
            4,
        ).await?
    };

    let pb = ProgressBar::new(100);
    pb.set_style(
        ProgressStyle::with_template(
            "[{elapsed_precise}] [{wide_bar:.cyan/blue}] {percent}%  {bytes_per_sec}  ETA {eta}",
        )
        .unwrap()
        .progress_chars("=> "),
    );

    let engine = TransferEngine::new(
        derive_stream_key(&sig_result.shared_key, 0),
        derive_control_key(&sig_result.shared_key),
    );

    match engine
        .receive(&out_dir, parallel, move |current, total, _| {
            if total > 0 {
                pb.set_length(total);
                pb.set_position(current);
            }
        })
        .await
    {
        Ok(_) => {
            println!("\x1b[32m✓ Transfer Complete! File(s) saved to: {}\x1b[0m", out_dir.display());
            Ok(())
        }
        Err(e) => {
            eprintln!(
                "ERROR: File corrupted during transfer or other error: {}",
                e
            );
            std::process::exit(1);
        }
    }
}
