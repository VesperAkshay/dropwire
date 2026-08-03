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
    _no_lan: bool,
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
    let relay_addr = tcp_relay
        .parse()
        .unwrap_or_else(|_| "127.0.0.1:9009".parse().unwrap());

    let parallel = ParallelStreams::connect(
        relay_addr,
        &channel_id,
        Role::Sender,
        &sig_result.auth_token,
        streams as u8,
    )
    .await?;

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

    engine
        .send(&file, parallel, move |current, total, _| {
            if total > 0 {
                pb.set_length(total);
                pb.set_position(current);
            }
        })
        .await?;

    let filesize = file.metadata().unwrap().len() as f64 / 1_048_576.0;
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
