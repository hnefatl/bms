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
    pub username: String,
    #[arg(long)]
    pub password_file: PathBuf,
}

