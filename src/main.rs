use clap::Parser;
use log::{error, info};
use rumqttc::{AsyncClient, ConnectionError, Event, EventLoop, MqttOptions, Packet, QoS};
use std::time::Duration;
use tokio::task::JoinSet;

mod args;
mod handler;
mod homeassistant;
mod ticker;
mod types;
use crate::handler::{Handler, HandlerArgs};

mod handlers {
    pub mod lights;
    pub mod medication;
}

const HANDLERS: [&'static dyn Handler; 1] = [
    &crate::handlers::medication::MEDICATION_REMINDER,
    //&crate::handlers::lights::HANDLER,
];

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    env_logger::init();
    let args = args::Args::parse();

    info!("Connecting to HomeAssistant on {}", &args.ha_url);
    let mut ha_client_builder = homeassistant::HAClientBuilder::new(args.ha_url, args.ha_token);

    let mut mqttoptions = MqttOptions::new("bms", &args.mqtt_host, args.mqtt_port);
    mqttoptions.set_keep_alive(Duration::from_secs(5));
    mqttoptions.set_credentials(&args.mqtt_username, &args.mqtt_password);
    mqttoptions.set_max_packet_size(1_000_000, 1_000_000);

    info!(
        "Connecting to MQTT on {}:{}",
        &args.mqtt_host, args.mqtt_port
    );
    let (mqtt_client, mut eventloop) = AsyncClient::new(mqttoptions, 256);

    if let Some(regex) = args.sniff_topics {
        mqtt_client.subscribe(".*", QoS::AtLeastOnce).await?;
        sniffer(regex, eventloop).await;
        return Ok(());
    }

    let (sender, receiver) = async_channel::bounded(256);
    let handler_args = HandlerArgs::new(mqtt_client.clone(), receiver);

    let mut joinset = JoinSet::new();
    for handler in HANDLERS {
        // TODO: better failure mode here? fallible initial setup step?
        for entity in handler.mqtt_entities() {
            info!("Subscribing to {}", entity.topic());
            mqtt_client
                .subscribe(entity.topic(), QoS::AtLeastOnce)
                .await
                .unwrap();
        }
        handler.start(&mut joinset, handler_args.clone(), &mut ha_client_builder);
    }

    let ha_client = ha_client_builder.start();

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
    joinset.abort_all();
    info!("Waiting for HA client to terminate...");
    ha_client.close().await;
    info!("Waiting for handlers to terminate...");
    while let Some(Err(_)) = joinset.join_next().await {}
    Ok(())
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
