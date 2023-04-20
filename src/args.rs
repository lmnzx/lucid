use clap::Parser;
use hyper::Uri;
use std::{ops::RangeInclusive, path::PathBuf};

#[derive(Parser, Debug, Clone)]
#[clap(author, version, about, long_about = None)]
pub struct Args {
    #[clap(default_value_t = Uri::from_static(&"http://127.0.0.1:8080/person"))]
    pub target_url: Uri,

    #[clap(short, long, default_value_t = 128, value_parser = connection_in_range)]
    pub connections: u16,

    #[clap(short, long, default_value_t = 1_0_000)]
    pub number_of_requests: u64,

    #[clap(short, long)]
    pub output_file: Option<PathBuf>,
}

// If client and server run on the same machine and both use the loopback interface,
// We must allow at most 2**16 -1 (one for the server) connections, since each connection requires a port.
// We stay away from the maximum by a margin of 10
// We do not allow to run with zero commands

const NUM_CON_RANGE: RangeInclusive<usize> = 1..=65536 - 10;
fn connection_in_range(s: &str) -> Result<u16, String> {
    s.parse()
        .iter()
        .filter(|i| NUM_CON_RANGE.contains(i))
        .map(|i| *i as u16)
        .next()
        .ok_or(format!(
            "Number of connection not in range {}-{}",
            NUM_CON_RANGE.start(),
            NUM_CON_RANGE.end()
        ))
}
