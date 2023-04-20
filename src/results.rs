use std::time::Duration;

#[derive(Debug)]
pub struct RequestResult {
    pub statuscode: u16,
    pub duration: Duration,
}
#[derive(Debug)]
pub enum RequestUpdate {
    Successful(RequestResult),
    Failure,
    Timeout,
}
