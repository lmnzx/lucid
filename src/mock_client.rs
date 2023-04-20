use anyhow::Result;
use async_trait::async_trait;
use hyper::Uri;
use rand::{thread_rng, RngCore};
use rand_distr::{Distribution, Normal};
use std::time::Duration;
pub struct WaitingHttpClient;

const CODE_DELAY_PAIRS: &[(u16, u16)] = &[
    (200, 20),
    (200, 30),
    (200, 40),
    (404, 20),
    (404, 100),
    (500, 200),
];

#[async_trait]
pub trait StatusOnlyHttpClient {
    async fn get(&self, uri: Uri) -> Result<u16>;
}

#[async_trait]
impl StatusOnlyHttpClient for WaitingHttpClient {
    async fn get(&self, _uri: Uri) -> Result<u16> {
        let (code, delay) = {
            let mut rng = thread_rng();
            let (code, base_delay) =
                CODE_DELAY_PAIRS[rng.next_u32() as usize % CODE_DELAY_PAIRS.len()];

            let normal = Normal::new(base_delay as f32, 5.0)?;
            (code, normal.sample(&mut rng))
        };
        spin_sleep::sleep(Duration::from_micros(delay as u64));
        Ok(code)
    }
}
