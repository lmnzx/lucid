use hyper::Uri;

#[derive(Clone)]
pub struct BenchmarkSettings {
    pub connections: u64,
    pub requests: u64,
    pub target_uri: Uri,
}

pub struct ConnectionSettings {
    pub connection_id: u64,
    pub target_uri: Uri,
    pub num_requests: u64,
}
