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
            if let Err(e) = dropwire::cli::send::run(file, code, streams, relay, no_lan).await {
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
            if let Err(e) = dropwire::cli::receive::run(code, out, relay, no_lan).await {
                eprintln!("Error: {}", e);
                std::process::exit(1);
            }
        }
        Commands::Relay { .. } => {
            eprintln!("Run the relay using the dropwire-relay binary.");
            std::process::exit(1);
        }
    }
}
