use hyper::Uri;

#[derive(Clone)]
pub struct BenchmarkSettings {
    pub connections: u64,
    pub target_uri: Uri,
    pub interval_ms: u64,
}

pub struct ConnectionSettings {
    pub connection_id: u64,
    pub target_uri: Uri,
    pub interval_ms: u64,
}
