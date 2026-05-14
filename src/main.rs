mod stream;
mod config;
mod logic_arbitrage;
mod logic_pricechange;
mod strategy;
mod notify;

use std::thread;
use std::time::Duration;
use crate::strategy::ArbitrageStrategy;
use crate::config::Mode;

use crate::logic_arbitrage::{TriangleArbitrage}; 
use crate::logic_pricechange::PriceChangeDetector;

fn main() {
    let cfg = config::Config::load();
    
    let pairs_to_stream = cfg.pairs.clone();

    let s = stream::create_stream(pairs_to_stream);

    let strategy: Box<dyn ArbitrageStrategy> = match cfg.mode {
        Mode::Triangle => {
            if cfg.pairs.len() < 3 {
                panic!("Need a 3 pairs!");
            }
            Box::new(TriangleArbitrage {
                p1: cfg.pairs[0].clone(),
                p2: cfg.pairs[1].clone(),
                p3: cfg.pairs[2].clone(),
                threshold: cfg.threshold,
                initial_asset: cfg.initial_asset.clone(),

            })
        },
        Mode::Pump | Mode::Dump => {
            Box::new(PriceChangeDetector {
                pairs: cfg.pairs.clone(),
                mode: cfg.mode,
                threshold: cfg.threshold,
                target_price: Some(cfg.target_price),
            })
        }
    };

    println!("Detector runs | Mode: {:?} | Count pairs: {}", cfg.mode, cfg.pairs.len());

    loop {
        if let Ok(data) = s.values.lock() {
            if let Some(msg) = strategy.analyze(&data) {
                strategy.alert(&msg);
            }
        }

        thread::sleep(Duration::from_millis(100));
    }
}
