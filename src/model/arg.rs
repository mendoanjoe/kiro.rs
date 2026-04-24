use clap::Parser;

/// Anthropic <-> Kiro API client
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Args {
    /// Configuration file path
    #[arg(short, long)]
    pub config: Option<String>,

    /// Credentials file path
    #[arg(long)]
    pub credentials: Option<String>,
}
