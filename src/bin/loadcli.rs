use hyper::Uri;
use lucid::{run, BenchmarkResults, BenchmarkSettings};
use std::{env, str::FromStr};

const DEFAULT_URL: &str = "http://127.0.0.1:8080/person";
const REQUESTS: u64 = 1_000_000;
const CONNECTIONS: u64 = 100;

#[tokio::main]
async fn main() {
    let uri = env::args()
        .nth(1)
        .map_or(Uri::from_static(DEFAULT_URL), |u| {
            Uri::from_str(&u).expect("Unparseable URI")
        });

    println!("Running angainst {uri}");

    let settings = BenchmarkSettings {
        connections: CONNECTIONS,
        requests: REQUESTS,
        target_uri: uri,
    };

    let BenchmarkResults {
        summaries,
        total_duration_ms: duration_ms,
    } = run(settings).await.expect("The benchmark failed:");

    for result in &summaries {
        println!(
            "con[{:4}]: ok: {}, ms: {}",
            result.connection_id,
            result.successful_requests,
            result.duration.as_millis()
        )
    }

    let ok_requests: u64 = summaries.iter().map(|r| r.successful_requests).sum();
    let failed_requests: u64 = summaries.iter().map(|r| r.failed_requests).sum();
    let max_duration = summaries
        .iter()
        .map(|r| r.duration.as_millis())
        .max()
        .unwrap() as u64;

    println!("Performed {ok_requests} ({failed_requests} failed) requests with a maximum of {max_duration}ms");

    println!("Sent {} requests in {}ms", REQUESTS, duration_ms);
    println!("This makes for{} req/s", REQUESTS * 1_000 / duration_ms);
}
