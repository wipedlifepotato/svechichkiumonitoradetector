mod stream;
mod config;
mod logic_arbitrage;
mod strategy;

use std::thread;
use std::time::Duration;

use crate::logic_arbitrage::TriangleArbitrage;
use crate::strategy::ArbitrageStrategy;

//use crate::stream;

fn main() {
	let cfg: config::Config = config::Config::load();
	
	let s = stream::create_stream_3(cfg.pairs[0], cfg.pairs[1], cfg.pairs[2]);//, cfg.best_choice);//VA_ARGS?
    //thread::sleep(Duration::from_secs(15));
	let s1 = stream::create_stream_3(cfg.pairs[2], cfg.pairs[1], cfg.pairs[0]);//, cfg.best_choice);//VA_ARGS?

    let strategy = TriangleArbitrage {
        p1: cfg.pairs[0].into(),
        p2: cfg.pairs[1].into(),
        p3: cfg.pairs[2].into(),
        threshold: -1.0//0.0005, // 0.05%
    };
    let strategy1 = TriangleArbitrage {
        p1: cfg.pairs[2].into(),
        p2: cfg.pairs[1].into(),
        p3: cfg.pairs[0].into(),
        threshold: -1.0//0.0005, // 0.05%
    };

    loop {
        if let Ok(data) = s.values.lock() {
            if let Some(msg) = strategy.analyze(&data) {
                strategy.alert(&msg);
            } else {
				//println!("Треуголки нет");
				//println!("Размер данных: {}", data.len());
				//dbg!(data);
			}
        }
        if let Ok(data) = s1.values.lock() {
            if let Some(msg) = strategy.analyze(&data) {
                strategy1.alert(&msg);
            } else {
				//println!("Треуголки нет");
				//println!("Размер данных: {}", data.len());
				//dbg!(data);
			}
        }
        thread::sleep(Duration::from_millis(100));
    }
	
	//if let Ok(data) = s.values.lock() {
	//		if let Some(eth_prices) = data.get("ETHBTC") {
	//			dbg!(eth_prices);
	//		}
	//}
    /*
    let market: Market = Binance::new(None, None);

    // Order book at default depth
    match market.get_depth("BTCUSD") {
        Ok(answer) => println!("{:?}", answer),
        Err(e) => println!("Error: {}", e),
    }

    println!("Hello, world!");
    */
    thread::sleep(Duration::from_secs(300));
}
