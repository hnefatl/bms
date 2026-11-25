use clap::Parser;

#[derive(Parser, Debug)]
#[command(version)]
pub struct Args {
    #[arg(long, default_value = "localhost")]
    pub mqtt_host: String,
    #[arg(long, default_value = "1883")]
    pub mqtt_port: u16,
    #[arg(long, default_value = "bms")]
    /// Username to authenticate to the MQTT broker with.
    pub mqtt_username: String,
    #[arg(long="mqtt-password-file", value_parser=parse_file_contents)]
    /// Path to a file containing the MQTT broker password for the username.
    pub mqtt_password: String,

    #[arg(long)]
    pub ha_url: String,
    #[arg(long="ha-token-file", value_parser=parse_file_contents)]
    pub ha_token: String,

    #[arg(long)]
    /// Print the payloads received from the broker, don't run normal operations.
    pub sniff_topics: Option<regex::Regex>,
}

fn parse_file_contents(value: &str) -> Result<String, clap::Error> {
    match std::fs::read_to_string(value) {
        Ok(c) => Ok(c.trim().to_string()),
        Err(e) => Err(e.into()),
    }
}
