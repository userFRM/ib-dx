//! The Rust example from the README, run against the venue.
//!
//! Kept as an example so it is compiled with everything else: a snippet in a
//! README that no longer builds is a snippet that turns readers away.

use std::time::{Duration, Instant};
use ib_dx::api::{Contract, Decimal, EClient, EClientConfig, TickAttrib, Wrapper};

struct App;

impl Wrapper for App {
    fn tick_price(&mut self, _req_id: i64, tick_type: i32, price: f64, _attrib: &TickAttrib) {
        println!("tick {tick_type}: {price}");
    }

    fn tick_size(&mut self, _req_id: i64, tick_type: i32, size: Decimal) {
        println!("size {tick_type}: {size}");
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = EClient::connect(&EClientConfig {
        username: std::env::var("IB_USERNAME").unwrap_or_default(),
        password: std::env::var("IB_PASSWORD").unwrap_or_default(),
        paper: true,
        ..Default::default()
    })?;

    let aapl = Contract {
        symbol: "AAPL".into(),
        sec_type: "STK".into(),
        exchange: "SMART".into(),
        currency: "USD".into(),
        ..Default::default()
    };
    client.req_mkt_data(1, &aapl, "", false, false, &[]);

    let mut app = App;
    let until = Instant::now() + Duration::from_secs(5);
    while Instant::now() < until {
        client.process_msgs(&mut app);
        std::thread::sleep(Duration::from_millis(50));
    }
    client.cancel_mkt_data(1);
    client.disconnect();
    Ok(())
}
