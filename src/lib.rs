use anyhow::{Context, Result};
use std::time::Instant;
use tokio::sync::mpsc::UnboundedSender;

mod mock_client;
pub mod results;
pub mod settings;

use mock_client::*;
use results::*;
use settings::*;

pub async fn run(
    settings: BenchmarkSettings,
    tx: UnboundedSender<BenchmarkUpdate>,
) -> Result<BenchmarkResults> {
    let mut handles = Vec::with_capacity(settings.connections as usize);
    let mut clients = Vec::with_capacity(settings.connections as usize);
    let mut all_results = Vec::with_capacity(settings.connections as usize);

    for _ in 0..settings.connections {
        let client = WaitingHttpClient;
        clients.push(client);
    }

    let start_instant = Instant::now();
    let number_of_connection_with_one_more_requests =
        (settings.requests % settings.connections) as usize;

    for (id, c) in clients.into_iter().enumerate() {
        let mut settings = ConnectionSettings {
            connection_id: id as u64,
            target_uri: settings.target_uri.clone(),
            num_requests: settings.requests / settings.connections,
        };
        if id < number_of_connection_with_one_more_requests {
            settings.num_requests += 1;
        }
        let h = tokio::spawn(connection_task(c, settings, tx.clone()));
        handles.push(h)
    }

    for h in handles {
        let await_result = h.await;
        let connection_result = await_result.context("Failed to await for task")?;
        let result = connection_result.context("A connection failed")?;
        all_results.push(result);
    }

    let duration_ms = start_instant.elapsed().as_millis() as u64;

    Ok(BenchmarkResults {
        summaries: all_results,
        total_duration_ms: duration_ms,
    })
}

async fn connection_task(
    client: impl StatusOnlyHttpClient,
    settings: ConnectionSettings,
    tx: UnboundedSender<BenchmarkUpdate>,
) -> Result<ConnectionSummary> {
    let mut summary = ConnectionSummary::new(settings.connection_id, settings.num_requests);
    let start_instant = Instant::now();
    for i in 0..settings.num_requests {
        do_request(&client, &settings, &mut summary, i, &tx).await?;
    }
    let duration = start_instant.elapsed();
    summary.duration = duration;
    Ok(summary)
}

async fn do_request(
    client: &impl StatusOnlyHttpClient,
    settings: &ConnectionSettings,
    summary: &mut ConnectionSummary,
    current_request: u64,
    tx: &UnboundedSender<BenchmarkUpdate>,
) -> Result<(), anyhow::Error> {
    let start_instant = Instant::now();
    let statuscode = client
        .get(settings.target_uri.clone())
        .await
        .context("A request failed")?;
    if statuscode < 408 {
        summary.successful_requests += 1;
    } else {
        summary.failed_requests += 1;
    }
    let duration = Instant::now().duration_since(start_instant);
    summary.requests.push(RequestResult {
        statuscode,
        duration,
    });
    if current_request % 100 == 0 {
        tx.send(BenchmarkUpdate {
            connection_id: settings.connection_id,
            current_request,
        })
        .context("The result channel was closed before the connection was done!")?;
    }
    Ok(())
}
