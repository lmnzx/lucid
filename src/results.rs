use std::time::Duration;

pub struct BenchmarkResults {
    pub summaries: Vec<ConnectionSummary>,
    pub total_duration_ms: u64,
}
#[derive(Debug)]
pub struct BenchmarkUpdate {
    pub connection_id: u64,
    pub current_request: u64,
}

pub struct ConnectionSummary {
    pub connection_id: u64,
    pub successful_requests: u64,
    pub failed_requests: u64,
    pub duration: Duration,
    pub requests: Vec<RequestResult>,
}

impl ConnectionSummary {
    pub fn new(connection_id: u64, number_of_requests: u64) -> Self {
        ConnectionSummary {
            connection_id,
            successful_requests: 0,
            failed_requests: 0,
            duration: Duration::default(),
            requests: Vec::with_capacity(number_of_requests as usize),
        }
    }
}

pub struct RequestResult {
    pub statuscode: u16,
    pub duration: Duration,
}
