use log::{error, info};
use rumqttc::{AsyncClient, ConnectionError, Event, EventLoop, MqttOptions, Packet, QoS};
use std::time::Duration;
use tokio::task::JoinSet;

mod args;
use args::*;
mod handler;
mod types;
use crate::handler::{Handler, HandlerArgs};

mod handlers {
    pub mod lights;
    pub mod medication;
}

const HANDLERS: [&'static dyn Handler; 1] = [
    &crate::handlers::medication::HANDLER,
    //&crate::handlers::lights::HANDLER,
];

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    env_logger::init();
    let args = args::Args::parse();
    let password = match std::fs::read_to_string(&args.password_file) {
        Ok(p) => p,
        Err(e) => {
            error!(
                "Failed to read password file '{}': {}",
                args.password_file.display(),
                e
            );
            Err(e)?
        }
    };

    let mut mqttoptions = MqttOptions::new("bms", &args.host, args.port);
    mqttoptions.set_keep_alive(Duration::from_secs(5));
    mqttoptions.set_credentials(args.username, password.trim());
    mqttoptions.set_max_packet_size(1_000_000, 1_000_000);

    info!("Connecting to {}:{}", &args.host, args.port);
    let (client, mut eventloop) = AsyncClient::new(mqttoptions, 256);

    if let Some(regex) = args.sniff_topics {
        client.subscribe(".*", QoS::AtLeastOnce).await?;
        sniffer(regex, eventloop).await;
        return Ok(());
    }

    let (sender, receiver) = async_channel::bounded(256);
    let handler_args = HandlerArgs::new(client.clone(), receiver);

    let mut joinset = JoinSet::new();
    for handler in HANDLERS {
        // TODO: better failure mode here? fallible initial setup step?
        for entity in handler.entities() {
            client
                .subscribe(entity.topic(), QoS::AtLeastOnce)
                .await
                .unwrap();
        }
        joinset.spawn(handler.handle(handler_args.clone()));
    }
    let watcher_handle = tokio::spawn(handler_watcher(joinset));

    loop {
        tokio::select! {
            _ = tokio::signal::ctrl_c() => {
                sender.close();
                break;
            }
            x = eventloop.poll() => match x {
                Ok(event) => {
                    sender.send(event).await.expect("ungraceful sender closure");
                }
                Err(ConnectionError::MqttState(e)) => {
                    // This is usually a transient error.
                    error!("MQTT state error: {}", e);
                }
                Err(e) => {
                    // This is usually a permanent error.
                    error!("MQTT error: {}", e);
                    tokio::time::sleep(Duration::from_secs(5)).await;
                }
            }
        }
    }
    info!("Waiting for handlers to terminate...");
    if let Err(e) = watcher_handle.await {
        error!("handler watcher failed to join: {}", e);
    }
    Ok(())
}

async fn handler_watcher(mut joinset: JoinSet<handler::Result<()>>) {
    while let Some(r) = joinset.join_next().await {
        // TODO: restart failed handlers
        match r {
            Ok(Ok(())) => {}
            Ok(Err(handler::Error::Shutdown)) => {}
            Ok(Err(e)) => error!("handler failed: {}", e),
            Err(e) => error!("handler failed to join: {}", e),
        }
    }
}

async fn sniffer(regex: regex::Regex, mut eventloop: EventLoop) {
    loop {
        match eventloop.poll().await {
            Ok(Event::Incoming(Packet::Publish(p))) => {
                if !regex.is_match(&p.topic) {
                    continue;
                }

                println!("topic '{}'", p.topic);
                match serde_json::from_slice::<serde_json::Value>(&p.payload) {
                    Ok(s) => println!("  payload: {}", s),
                    Err(e) => println!("  decoding error in '{:?}': {}", p.payload, e),
                }
                println!();
            }
            Ok(Event::Outgoing(_)) => {}
            Ok(e) => println!("{:?}", e),
            Err(e) => {
                println!("error: {}", e);
            }
        }
    }
}
