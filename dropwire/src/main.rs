use clap::Parser;
use dropwire::cli::{Cli, Commands};

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let cli = Cli::parse();

    match cli.command {
        Commands::Send {
            file,
            code,
            streams,
            relay,
            no_lan,
        } => {
            let config = dropwire::cli::config::DropwireConfig::load();
            let final_relay = relay.unwrap_or_else(|| config.get_relay());
            if let Err(e) = dropwire::cli::send::run(file, code, streams, final_relay, no_lan).await {
                eprintln!("Error: {}", e);
                std::process::exit(1);
            }
        }
        Commands::Receive {
            code,
            out,
            relay,
            no_lan,
        } => {
            let config = dropwire::cli::config::DropwireConfig::load();
            let final_relay = relay.unwrap_or_else(|| config.get_relay());
            if let Err(e) = dropwire::cli::receive::run(code, out, final_relay, no_lan).await {
                eprintln!("Error: {}", e);
                std::process::exit(1);
            }
        }
        Commands::Relay { .. } => {
            eprintln!("Run the relay using the dropwire-relay binary.");
            std::process::exit(1);
        }
        Commands::Config { action, key, value } => {
            if action.to_lowercase() == "show" {
                dropwire::cli::config::run_show();
            } else if action.to_lowercase() == "set" {
                if let (Some(k), Some(v)) = (key, value) {
                    dropwire::cli::config::run_set(k, v);
                } else {
                    eprintln!("Usage: dropwire config set <key> <value>");
                }
            } else {
                eprintln!("Unknown config action. Use 'show' or 'set'.");
            }
        }
    }
}
