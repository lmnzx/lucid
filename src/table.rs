use serde::Serialize;
use statrs::statistics::{Data, Distribution, Max, Min, OrderStatistics};
use tabled::{self, Tabled};

const MICROS_PER_SEC: f64 = 1_000_000.0;

#[derive(Tabled, Serialize)]
pub struct ResultTableEntry {
    pub status_code: u16,
    pub observations: u32,
    #[tabled(display_with = "two_digit_float")]
    pub average_rate: f64,
    #[tabled(display_with = "two_digit_float")]
    mean: f64,
    #[tabled(display_with = "two_digit_float")]
    min: f64,
    #[tabled(display_with = "two_digit_float")]
    max: f64,
    #[tabled(display_with = "two_digit_float")]
    sd: f64,
    #[tabled(display_with = "two_digit_float")]
    p90: f64,
    #[tabled(display_with = "two_digit_float")]
    p99: f64,
}

impl ResultTableEntry {
    pub fn new(status_code: u16, mut data: Data<Vec<f64>>) -> Self {
        ResultTableEntry {
            status_code,
            observations: data.len() as u32,
            average_rate: MICROS_PER_SEC / data.mean().unwrap_or_default(),
            mean: data.mean().unwrap_or_default(),
            min: data.min(),
            max: data.max(),
            sd: data.std_dev().unwrap_or_default(),
            p90: data.percentile(90),
            p99: data.percentile(99),
        }
    }
}

fn two_digit_float(v: &f64) -> String {
    format! {"{:.2}",v}
}
