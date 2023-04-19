use anyhow::{Context, Result};
use hyper::{client::HttpConnector, Body, Client, Uri};
use std::time::{Duration, Instant};
type HttpClient = Client<HttpConnector, Body>;
use async_trait::async_trait;

pub struct BenchmarkSettings {
    pub connections: u64,
    pub requests: u64,
    pub target_uri: Uri,
}
pub struct BenchmarkResults {
    pub summaries: Vec<ConnectionSummary>,
    pub total_duration_ms: u64,
}

pub struct ConnectionSummary {
    pub connection_id: u64,
    pub successful_requests: u64,
    pub failed_requests: u64,
    pub duration: Duration,
}
impl ConnectionSummary {
    fn new(connection_id: u64) -> Self {
        ConnectionSummary {
            connection_id,
            successful_requests: 0,
            failed_requests: 0,
            duration: Duration::default(),
        }
    }
}

struct ConnectionSettings {
    connection_id: u64,
    target_uri: Uri,
    num_requests: u64,
}

pub async fn run(settings: BenchmarkSettings) -> Result<BenchmarkResults> {
    let mut handles = Vec::with_capacity(settings.connections as usize);
    let mut clients = Vec::with_capacity(settings.connections as usize);
    let mut all_results = Vec::with_capacity(settings.connections as usize);

    for _ in 0..settings.connections {
        let client = Client::builder().build_http();
        clients.push(client);
    }

    let start_instant = Instant::now();

    for (id, c) in clients.into_iter().enumerate() {
        let settings = ConnectionSettings {
            connection_id: id as u64,
            target_uri: settings.target_uri.clone(),
            num_requests: settings.requests / settings.connections,
        };
        let h = tokio::spawn(connection_task(c, settings));
        handles.push(h)
    }

    for h in handles {
        let await_result = h.await;
        let connection_result = await_result.context("Failed to await for task")?;
        let result = connection_result.context("A connection failed")?;
        all_results.push(result);
    }

    let duration_ms = start_instant.elapsed().as_millis() as u64;

    Ok(BenchmarkResults {
        summaries: all_results,
        total_duration_ms: duration_ms,
    })
}

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

async fn connection_task(
    client: impl StatusOnlyHttpClient,
    settings: ConnectionSettings,
) -> Result<ConnectionSummary> {
    let mut summary = ConnectionSummary::new(settings.connection_id);
    let start_instant = Instant::now();
    for _ in 0..settings.num_requests {
        let status = client
            .get(settings.target_uri.clone())
            .await
            .context("A request failed")?; // don't worry, we take care of this shortly
        if status < 408 {
            summary.successful_requests += 1;
        } else {
            summary.failed_requests += 1;
        }
    }
    let duration = start_instant.elapsed();
    summary.duration = duration;
    Ok(summary)
}

#[cfg(test)]
mod tests {
    use anyhow::{anyhow, Result};
    use async_trait::async_trait;
    use hyper::Uri;

    use crate::{connection_task, ConnectionSettings, StatusOnlyHttpClient};

    struct MockHttpClient {
        fixed_status_response: Option<u16>,
    }

    impl MockHttpClient {
        fn with_result(result: Option<u16>) -> Self {
            MockHttpClient {
                fixed_status_response: result,
            }
        }
    }

    #[async_trait]
    impl StatusOnlyHttpClient for MockHttpClient {
        async fn get(&self, _uri: Uri) -> Result<u16> {
            match self.fixed_status_response {
                Some(status) => Ok(status),
                None => Err(anyhow!("SomeError")),
            }
        }
    }

    #[tokio::test]
    async fn test_happy_path() {
        let client = MockHttpClient::with_result(Some(200));
        let res = connection_task(client, common_settings()).await;
        let res = res.expect("do not expect a result");
        assert_eq!(res.successful_requests, 10);
        assert_eq!(res.failed_requests, 0);
    }

    #[tokio::test]
    async fn test_with_bad_status_code() {
        let client = MockHttpClient::with_result(Some(500));
        let res = connection_task(client, common_settings()).await;
        let res = res.expect("do not expect a result");
        assert_eq!(res.successful_requests, 0);
        assert_eq!(res.failed_requests, 10);
    }

    #[tokio::test]
    async fn test_with_error() {
        let client = MockHttpClient::with_result(None);
        let res = connection_task(client, common_settings()).await;
        assert_eq!(res.is_err(), true);
    }

    fn common_settings() -> ConnectionSettings {
        ConnectionSettings {
            connection_id: 0,
            target_uri: Uri::from_static("http://dummy"),
            num_requests: 10,
        }
    }
}
