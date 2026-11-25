use std::{collections::HashMap, sync::Arc, time::Duration};

use hass_rs::{self, HassClient, HassError, HassEvent, WSEvent};
use log::{debug, error, info, warn};
use tokio::{
    sync::{Mutex, mpsc},
    task::JoinHandle,
    time::{interval, timeout},
};
use tokio_util::task::TaskTracker;

const STATE_CHANGED_EVENT: &'static str = "state_changed";

/// Consume events from the `HassClient` stream and forward them to the persistent channel.
/// This is designed to be fired up for a client then aborted when the client disconnects.
async fn forward_events(mut receiver: mpsc::Receiver<WSEvent>, sender: mpsc::Sender<WSEvent>) {
    // Keep forwarding until either channel closes: if the receiver closes,
    // the client disconnected so a new listener will be spawned. If the
    // sender closed, the `HAClient` is getting dropped.
    while let Some(e) = receiver.recv().await
        && let Ok(()) = sender.send(e).await
    {}
}

async fn handle_subscriptions(
    mut receiver: mpsc::Receiver<WSEvent>,
    subscriptions: HashMap<String, Vec<mpsc::Sender<HassEvent>>>,
) {
    while let Some(e) = receiver.recv().await {
        debug!("Received WSEvent: {:?}", e);
        if let Some(entity_id) = &e.event.data.entity_id
            && let Some(senders) = subscriptions.get(entity_id)
        {
            for sender in senders {
                let _ = sender.send(e.event.clone()).await;
            }
        };
    }
}

pub struct HAClientBuilder {
    state: State,
    subscriptions: HashMap<String, Vec<mpsc::Sender<HassEvent>>>,
    receiver: mpsc::Receiver<WSEvent>,
}
impl HAClientBuilder {
    pub fn new(url: String, token: String) -> Self {
        let (s, r) = mpsc::channel(256);
        Self {
            state: State {
                url,
                token,
                client: Mutex::new(None),
                event_sender: s,
                tasks: TaskTracker::new(),
            },
            subscriptions: HashMap::new(),
            receiver: r,
        }
    }

    pub fn subscribe<S: Into<String>>(&mut self, entity_id: S) -> mpsc::Receiver<HassEvent> {
        let (s, r) = mpsc::channel(256);
        self.subscriptions
            .entry(entity_id.into())
            .or_default()
            .push(s);
        r
    }

    pub fn start(self) -> HAClient {
        debug!("HAClient started with subscriptions: {:?}", &self.subscriptions.keys());
        let state = Arc::new(self.state);
        HAClient {
            state: state.clone(),
            keepalive_handle: tokio::spawn(async move { state.keepalive().await }),
            event_reader_handle: tokio::spawn(async move {
                handle_subscriptions(self.receiver, self.subscriptions).await
            }),
        }
    }
}

/// A reconnecting opinionated client for HA.
pub struct HAClient {
    /// Arc to allow sharing this with the keepalive thread.
    state: Arc<State>,

    keepalive_handle: JoinHandle<!>,
    event_reader_handle: JoinHandle<()>,
}
impl HAClient {
    /// Shutdown and wait for all tasks to finish/abort.
    pub async fn close(self) {
        self.event_reader_handle.abort();
        self.keepalive_handle.abort();
        self.state.close().await;
        let _ = self.keepalive_handle.await;
    }
}

struct State {
    url: String,
    token: String,
    client: Mutex<Option<HassClient>>,
    event_sender: mpsc::Sender<WSEvent>,
    tasks: TaskTracker,
}
impl State {
    /// Must be left running to allow the client to handle reconnects.
    async fn keepalive(&self) -> ! {
        let mut ticker = interval(Duration::from_secs(10));
        loop {
            self.ensure_connected().await;
            ticker.tick().await;
        }
    }

    async fn ensure_connected(&self) {
        let mut guard = self.client.lock().await;
        if let Some(c) = guard.as_mut()
            && try_ping(c).await
        {
            // We have a client and pinging works, we're connected.
            return;
        };

        match self.try_connect().await {
            Ok((client, receiver)) => {
                info!("Connected to HA");
                // Start a listener for this client.
                self.tasks
                    .spawn(forward_events(receiver, self.event_sender.clone()));
                *guard = Some(client);
            }
            Err(e) => {
                *guard = None;
                error!("Disconnected from  to HA: {}", e);
            }
        }
    }

    async fn try_connect(&self) -> Result<(HassClient, mpsc::Receiver<WSEvent>), HassError> {
        let mut client = HassClient::new(&self.url).await?;
        client.auth_with_longlivedtoken(&self.token).await?;
        let receiver = client.subscribe_event(STATE_CHANGED_EVENT).await?;
        Ok((client, receiver))
    }

    async fn close(&self) {
        self.tasks.close();
        self.tasks.wait().await;
    }
}

async fn try_ping(c: &mut HassClient) -> bool {
    matches!(timeout(Duration::from_secs(30), c.ping()).await, Ok(Ok(())))
}
