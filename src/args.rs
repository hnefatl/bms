pub use clap::Parser;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(version)]
pub struct Args {
    #[arg(long, default_value = "localhost")]
    pub host: String,
    #[arg(long, default_value = "1883")]
    pub port: u16,
    #[arg(long, default_value = "bms")]
    /// Username to authenticate to the MQTT broker with.
    pub username: String,
    #[arg(long)]
    /// Path to a file containing the MQTT broker password for the username.
    pub password_file: PathBuf,

    #[arg(long)]
    /// Print the payloads received from the broker, don't run normal operations.
    pub sniff_topics: Option<regex::Regex>,
}

