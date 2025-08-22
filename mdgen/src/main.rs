use mdgen::generator::MarketDataGenerator;
use std::time::Duration;

fn main() {
    let mut generator = MarketDataGenerator::new(100.0, 0.2, 0.01);
    let simulation_duration = Duration::from_secs(60);
    let data_interval = Duration::from_millis(100);

    println!("timestamp,type,price,size,bid_px,bid_sz,ask_px,ask_sz");

    let start_time = std::time::Instant::now();
    while std::time::Instant::now() - start_time < simulation_duration {
        if let Some(data) = generator.next() {
            match data {
                mdgen::models::MarketData::Quote(q) => {
                    println!(
                        "{},QUOTE,,,,{},{},{},{}",
                        q.timestamp, q.bid_px, q.bid_sz, q.ask_px, q.ask_sz
                    );
                }
                mdgen::models::MarketData::Trade(t) => {
                    println!("{},TRADE,{},{},,,,", t.timestamp, t.px, t.sz);
                }
            }
        }
        std::thread::sleep(data_interval);
    }
}