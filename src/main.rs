use std::time::Duration;

use clap::Parser;
use tokio::{
    select,
    sync::mpsc::{self, UnboundedReceiver},
    time::interval,
};

use lucid::{results::*, run, settings::*};

mod args;

use crate::args::Args;

#[tokio::main]
async fn main() {
    let args = Args::parse();

    let (terminate_tx, terminate_rx) = tokio::sync::watch::channel(true);

    ctrlc::set_handler(move || {
        terminate_tx
            .send(false)
            .expect("Could not send signal on channel.")
    })
    .expect("Error setting Ctrl-C handler");

    let (tx, rx) = mpsc::unbounded_channel::<RequestUpdate>();
    let output_handle = tokio::spawn(live_output(args.clone(), rx));

    let settings = BenchmarkSettings {
        connections: args.connections as u64,
        interval_ms: args.interval_ms,
        target_uri: args.target_url,
    };

    println!("Running against {}", settings.target_uri);

    // first wait for all connections to close before we close the output component
    run(settings.clone(), tx, terminate_rx).await;
    let _ = output_handle.await;
}

async fn live_output(_args: Args, mut rx: UnboundedReceiver<RequestUpdate>) {
    let print_interval = 2000;

    let mut observation_count = 0;
    let mut aggregated_latency_us = 0;
    let mut last_observation_count = 0;
    let mut last_aggregated_latency_us = 0;
    let mut interval = interval(Duration::from_millis(print_interval));

    loop {
        select! {
            _ = interval.tick() => {
                // println!("Ticked")
                let amount = observation_count-last_observation_count;
                if amount>0{
                    let latency_sum = aggregated_latency_us-last_aggregated_latency_us;
                    let average_latency = latency_sum/amount;
                    println!("average_latency: {average_latency}us");
                }
                last_observation_count = observation_count;
                last_aggregated_latency_us = aggregated_latency_us;
            }
            update = rx.recv() => {
                match update{
                    Some(update) => {
                        match update{
                            RequestUpdate::Successful(res) => {
                                println!("Observed request: {:?}",res);
                                observation_count += 1;
                                aggregated_latency_us += res.duration.as_micros();
                            },
                            RequestUpdate::Failure => {
                                println!("Observed failure: {:?}",update);
                            },
                            RequestUpdate::Timeout => {
                                println!("Observed timeout: {:?}",update);
                            }
                        }

                    }
                    None =>{
                        println!("Terminating printer");
                        return
                    }
                }
            }
        }
    }
}
