use anyhow::Result;
use async_trait::async_trait;
use hyper::{client::HttpConnector, Body, Client, Uri};
use std::time::{Duration, Instant};
use tokio::{
    select,
    sync::{mpsc::UnboundedSender, watch::Receiver},
    time::{interval, timeout},
};

pub mod results;
pub mod settings;

use results::*;
use settings::*;

type HttpClient = Client<HttpConnector, Body>;

#[async_trait]
pub trait StatusOnlyHttpClient {
    async fn get(&self, uri: Uri) -> Result<u16>;
}

#[async_trait]
impl StatusOnlyHttpClient for HttpClient {
    async fn get(&self, uri: Uri) -> Result<u16> {
        let status = self.get(uri).await.map(|r| r.status().as_u16())?;
        Ok(status)
    }
}

pub async fn run(
    settings: BenchmarkSettings,
    update_tx: UnboundedSender<RequestUpdate>,
    terminate_rx: Receiver<bool>,
) {
    let _handles = (0..settings.connections)
        .into_iter()
        .map(|id| {
            let settings = ConnectionSettings {
                connection_id: id as u64,
                target_uri: settings.target_uri.clone(),
                interval_ms: settings.interval_ms,
            };
            tokio::spawn(connection_task(
                Client::builder().build_http(),
                settings,
                update_tx.clone(),
                terminate_rx.clone(),
            ))
        })
        .collect::<Vec<_>>();
}

async fn connection_task(
    client: impl StatusOnlyHttpClient,
    settings: ConnectionSettings,
    tx: UnboundedSender<RequestUpdate>,
    mut terminate_rx: Receiver<bool>,
) {
    let mut interval = interval(Duration::from_millis(settings.interval_ms));
    while let Ok(false) = terminate_rx.has_changed() {
        do_request(&client, &settings, &tx).await;
        select! {
            _ = interval.tick() => {
                // just continue
            }
            _ = terminate_rx.changed() => {
                break;
            }
        };
    }
    println!("Terminating connection {}", settings.connection_id);
}

async fn do_request(
    client: &impl StatusOnlyHttpClient,
    settings: &ConnectionSettings,
    tx: &UnboundedSender<RequestUpdate>,
) {
    let request_future = client.get(settings.target_uri.clone());

    let start_instant = Instant::now();
    let result = timeout(Duration::from_millis(500), request_future).await;
    let duration = Instant::now().duration_since(start_instant);

    let update = match result {
        Ok(Ok(statuscode)) => RequestUpdate::Successful(RequestResult {
            statuscode,
            duration,
        }),
        Ok(Err(_)) => RequestUpdate::Failure,
        Err(_) => RequestUpdate::Timeout,
    };

    tx.send(update)
        .expect("The result channel was closed before the connection was done!");
}
