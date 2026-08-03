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
    _no_lan: bool,
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
    let relay_addr = tcp_relay
        .parse()
        .unwrap_or_else(|_| "127.0.0.1:9009".parse().unwrap());

    let parallel = ParallelStreams::connect(
        relay_addr,
        &channel_id,
        Role::Receiver,
        &sig_result.auth_token,
        4, // The receiver determines stream count by what it receives, but ParallelStreams expects a number
    )
    .await?;

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
            println!("Received to output directory");
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
