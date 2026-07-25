use clap::Parser;
use rumqttc::{AsyncClient, MqttOptions, QoS};
use std::{path::PathBuf, time::Duration};

#[derive(Parser, Debug)]
#[command(version)]
struct Args {
    #[arg(long, default_value = "localhost")]
    host: String,
    #[arg(long, default_value = "1883")]
    port: u16,
    #[arg(long, default_value = "bms")]
    username: String,
    #[arg(long)]
    password_file: PathBuf,
}

#[tokio::main]
async fn main() {
    let args = Args::parse();
    let password =
        std::fs::read_to_string(args.password_file).expect("failed to read password file");

    let mut mqttoptions = MqttOptions::new("bms", args.host, args.port);
    mqttoptions.set_keep_alive(Duration::from_secs(5));
    mqttoptions.set_credentials(args.username, password.trim());

    let (client, mut eventloop) = AsyncClient::new(mqttoptions, 100);
    client
        .subscribe("homeassistant/#", QoS::AtMostOnce)
        .await
        .unwrap();

    loop {
        match eventloop.poll().await {
            Ok(notification) => println!("Received: {:?}", notification),
            Err(e) => {
                println!("Error: {}", e);
                tokio::time::sleep(Duration::from_secs(5)).await;
            }
        }
    }
}
