use std::{collections::HashMap, fs::File, path::PathBuf, time::Duration};

use anyhow::{bail, Context, Result};
use clap::Parser;
use indicatif::{ProgressBar, ProgressStyle};
use lucid::{results::*, run, settings::*};
use statrs::statistics::Data;
use tabled::TableIteratorExt;
use tokio::sync::mpsc::{self, UnboundedReceiver};

mod args;
mod table;

use crate::args::Args;
use crate::table::*;

#[tokio::main]
async fn main() {
    let args = Args::parse();

    let (tx, rx) = mpsc::unbounded_channel::<BenchmarkUpdate>();
    let output_handle = tokio::spawn(live_output(args.clone(), rx));

    let settings = BenchmarkSettings {
        connections: args.connections as u64,
        requests: args.number_of_requests,
        target_uri: args.target_url,
    };

    println!("Running against {}", settings.target_uri);

    let result = run(settings.clone(), tx)
        .await
        .expect("The benchmark failed:");
    let _ = output_handle.await;

    print_metadata(&settings, &result);
    let data = tabular_data(&result);

    print_table(&data);
    if let Some(output_file) = args.output_file {
        write_csv(&output_file, &data).expect("Could not write output file");
    }
}

fn print_metadata(settings: &BenchmarkSettings, results: &BenchmarkResults) {
    let ok_requests: u64 = results
        .summaries
        .iter()
        .map(|r| r.successful_requests)
        .sum();
    let failed_requests: u64 = results.summaries.iter().map(|r| r.failed_requests).sum();

    println!(
        "Sent {} requests in {}ms to {} from {} connections",
        settings.requests, results.total_duration_ms, settings.target_uri, settings.connections
    );
    println!("Performed {ok_requests} ({failed_requests} failed) requests.");
}

fn tabular_data(results: &BenchmarkResults) -> Vec<ResultTableEntry> {
    let mut microseconds_by_status_code: HashMap<u16, Vec<f64>> = HashMap::new();

    results
        .summaries
        .iter()
        .map(|s| &s.requests)
        .flatten()
        .for_each(|r| {
            microseconds_by_status_code
                .entry(r.statuscode)
                .or_default()
                .push(r.duration.as_micros() as f64)
        });

    microseconds_by_status_code
        .into_iter()
        .map(|(k, v)| (k, Data::new(v)))
        .map(|(k, duration)| ResultTableEntry::new(k, duration))
        .collect::<Vec<ResultTableEntry>>()
}

fn print_table(data: &Vec<ResultTableEntry>) {
    let t = data.table();
    println!("{}", t);
}

fn write_csv(path: &PathBuf, data: &Vec<ResultTableEntry>) -> Result<()> {
    println!(
        "Saving results to {}",
        path.to_str().context("Path cannot even be displayed")?
    );

    if path.exists() {
        bail!("File exists and won't be touched");
    }

    let file = File::create(path).context("Could not create output file")?;

    let mut wtr = csv::Writer::from_writer(file);
    for line in data {
        wtr.serialize(line)
            .context("Could noy write to structured file")?;
    }
    wtr.flush().context("Could not flush structured file")?;
    Ok(())
}

async fn live_output(args: Args, mut rx: UnboundedReceiver<BenchmarkUpdate>) {
    let pb = ProgressBar::new(args.number_of_requests as u64);
    pb.set_style(
        ProgressStyle::with_template(
            "{spinner:.green} [{elapsed_precise}] [{bar:.blue}] {per_sec:7}",
        )
        .unwrap(),
    );
    pb.enable_steady_tick(Duration::from_millis(100));
    let mut accumulated_connections = vec![0u64; args.connections.into()];
    while let Some(update) = rx.recv().await {
        accumulated_connections[update.connection_id as usize] = update.current_request;
        pb.set_position(accumulated_connections.iter().sum());
    }
    pb.finish_and_clear();
}
