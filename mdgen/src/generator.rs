use crate::models::{current_timestamp_ns, MarketData, Quote, Trade};
use rand::prelude::*;
use rand_distr::{Distribution, StandardNormal};
use rand_chacha::ChaCha8Rng; 

pub struct MarketDataGenerator {
    price: f64,
    volatility: f64,
    dt: f64, 
    rng: ChaCha8Rng, 
}

impl MarketDataGenerator {
    pub fn new(initial_price: f64, volatility: f64, dt: f64) -> Self {
        Self {
            price: initial_price,
            volatility,
            dt,
            rng: ChaCha8Rng::from_entropy(), 
        }
    }

    fn update_price(&mut self) {
        let drift = 0.0; 
        let random_component: f64 = StandardNormal.sample(&mut self.rng);
        let diffusion = self.volatility * random_component * self.dt.sqrt();
        let price_change = self.price * (drift * self.dt + diffusion);
        self.price += price_change;
    }
}

impl Iterator for MarketDataGenerator {
    type Item = MarketData;

    fn next(&mut self) -> Option<Self::Item> {
        self.update_price();

        let event_type: u8 = self.rng.gen_range(0..=4);

        match event_type {
            0..=3 => {
                let spread = self.price * 0.001; 
                let quote = Quote {
                    timestamp: current_timestamp_ns(),
                    bid_px: self.price - spread / 2.0,
                    bid_sz: self.rng.gen_range(1..=10) * 100,
                    ask_px: self.price + spread / 2.0,
                    ask_sz: self.rng.gen_range(1..=10) * 100,
                };
                Some(MarketData::Quote(quote))
            }
            4 => {
                let trade = Trade {
                    timestamp: current_timestamp_ns(),
                    px: self.price,
                    sz: self.rng.gen_range(1..=5) * 10,
                };
                Some(MarketData::Trade(trade))
            }
            _ => None,
        }
    }
}