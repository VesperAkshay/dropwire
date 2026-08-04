use crate::crypto::kdf::{derive_control_key, derive_stream_key};
use crate::error::DropWireError;
use crate::net::parallel::ParallelStreams;
use crate::net::signaling::{Role, SignalingClient};
use crate::transfer::engine::TransferEngine;
use crate::types::ChannelId;
use indicatif::{ProgressBar, ProgressStyle};
use rand::Rng;
use std::path::PathBuf;
pub async fn run(
    file: PathBuf,
    code: Option<String>,
    streams: usize,
    relay_url: String,
    no_lan: bool,
) -> Result<(), DropWireError> {
    if !file.exists() {
        eprintln!("Error: Path does not exist.");
        std::process::exit(1);
    }

    let code_phrase = code.unwrap_or_else(generate_code);
    let channel_id = ChannelId::derive(&code_phrase);

    println!("Code: {}", code_phrase);
    println!("Waiting for peer...");

    // For Phase 9, we simplify by using Signaling to relay.
    let sig_result =
        SignalingClient::connect(&relay_url, &channel_id, Role::Sender, &code_phrase).await?;

    // Wait a moment for peer to be ready if needed
    // Connect ParallelStreams
    // We connect to relay port, wait, signaling connect_inner uses relay_ws_url which is 9010.
    // The TCP relay is at 9009.
    let tcp_relay = relay_url.replace("ws://", "").replace("9010", "9009");
    let parallel = if !no_lan {
        let listener = crate::net::parallel::ParallelListener::bind().await?;
        let port = listener.port;
        let chan = channel_id.clone();
        println!("\x1b[36m⚡ LAN Discovery: Listening on port {} ...\x1b[0m", port);
        tokio::spawn(async move {
            let _ = crate::net::discovery::DiscoveryService::announce(&chan, std::time::Duration::from_secs(30), port).await;
        });

        match tokio::time::timeout(
            std::time::Duration::from_secs(15),
            listener.accept_all(&sig_result.auth_token, streams as u8)
        ).await {
            Ok(Ok(p)) => {
                println!("\x1b[32m✓ Connected via LAN (Direct P2P) — Maximum speed!\x1b[0m");
                p
            }
            _ => {
                println!("\x1b[33m⚠ LAN peer not found, falling back to Relay...\x1b[0m");
                ParallelStreams::connect(
                    tcp_relay.clone(),
                    &channel_id,
                    Role::Sender,
                    &sig_result.auth_token,
                    streams as u8,
                ).await?
            }
        }
    } else {
        println!("\x1b[33m⚠ LAN disabled by config, using Relay...\x1b[0m");
        ParallelStreams::connect(
            tcp_relay,
            &channel_id,
            Role::Sender,
            &sig_result.auth_token,
            streams as u8,
        ).await?
    };

    let pb = ProgressBar::new(100); // 100% based, but we'll use ETA logic
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

    let config = crate::cli::config::DropwireConfig::load();
    
    engine
        .send(&[file.to_path_buf()], parallel, config.get_chunk_size_kb(), move |current, total, _| {
            if total > 0 {
                pb.set_length(total);
                pb.set_position(current);
            }
        })
        .await?;

    let filesize = if file.is_dir() {
        walkdir::WalkDir::new(&file)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().is_file())
            .map(|e| e.metadata().map(|m| m.len()).unwrap_or(0))
            .sum::<u64>() as f64 / 1_048_576.0
    } else {
        file.metadata().unwrap().len() as f64 / 1_048_576.0
    };
    println!(
        "Transfer complete: {} ({:.2} MB)",
        file.file_name().unwrap().to_string_lossy(),
        filesize
    );

    Ok(())
}

fn generate_code() -> String {
    let words = &crate::wordlist::BIP39_WORDS;
    let mut rng = rand::thread_rng();

    let num = rng.gen_range(1..=999);
    let word1 = words[rng.gen_range(0..words.len())];
    let word2 = words[rng.gen_range(0..words.len())];

    format!("{}-{}-{}", num, word1, word2)
}
