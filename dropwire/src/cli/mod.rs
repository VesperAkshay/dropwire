use clap::{Parser, Subcommand};
use std::path::PathBuf;

pub mod receive;
pub mod relay;
pub mod send;

#[derive(Parser, Debug)]
#[command(
    name = "dropwire", 
    bin_name = "dropwire",
    version = "1.0", 
    about = "Dropwire: Secure, fast, P2P file transfer",
    long_about = "Dropwire is a high-speed, end-to-end encrypted P2P file transfer tool.\n\nIt allows you to securely send files across any network (LAN or WAN) using SPAKE2 password-authenticated key exchange and parallel TCP streams for maximum throughput.\n\nQUICK START:\n  Sender:   dropwire send ./my_file.zip\n  Receiver: dropwire receive <CODE>"
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug, PartialEq)]
pub enum Commands {
    /// Send a file securely to another peer
    Send {
        /// File to send
        file: PathBuf,
        /// Use specific code instead of random
        #[arg(short, long)]
        code: Option<String>,
        /// Number of parallel streams
        #[arg(short, long, default_value_t = 4)]
        streams: usize,
        /// Relay address
        #[arg(short, long, default_value = "ws://relay.dropwire.io:9010")]
        relay: String,
        /// Skip LAN discovery
        #[arg(long)]
        no_lan: bool,
    },
    /// Receive a file
    Receive {
        /// Code phrase
        code: String,
        /// Output directory (defaults to ~/Downloads/Dropwire)
        #[arg(short, long)]
        out: Option<PathBuf>,
        /// Relay address
        #[arg(short, long, default_value = "ws://relay.dropwire.io:9010")]
        relay: String,
        /// Skip LAN discovery
        #[arg(long)]
        no_lan: bool,
    },
    /// Run a relay server
    Relay {
        /// TCP bind address
        #[arg(long, default_value = "0.0.0.0:9009")]
        bind: String,
        /// WS bind address
        #[arg(long, default_value = "0.0.0.0:9010")]
        ws_bind: String,
    },
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::CommandFactory;

    #[test]
    fn test_cli_send() {
        let cli = Cli::parse_from(["dropwire", "send", "test.txt", "--streams", "8"]);
        match cli.command {
            Commands::Send { file, streams, .. } => {
                assert_eq!(file.to_str().unwrap(), "test.txt");
                assert_eq!(streams, 8);
            }
            _ => panic!("Expected Send command"),
        }
    }

    #[test]
    fn test_cli_receive() {
        let cli = Cli::parse_from(["dropwire", "receive", "7-guitar-revenge", "--out", "."]);
        match cli.command {
            Commands::Receive { code, out, .. } => {
                assert_eq!(code, "7-guitar-revenge");
                assert_eq!(out.unwrap().to_str().unwrap(), ".");
            }
            _ => panic!("Expected Receive command"),
        }
    }

    #[test]
    fn test_help() {
        Cli::command().debug_assert();
    }
}
