mod stream;
use std::thread;
use std::time::Duration;
mod config;
//use crate::stream;
fn main() {
	let cfg: config::Config = config::Config::load();
	
	let s = stream::Create_stream_3(cfg.pairs[0], cfg.pairs[1], cfg.pairs[2]);//, cfg.best_choice);//VA_ARGS?

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
