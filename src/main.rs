mod stream;
use std::thread;
use std::time::Duration;
//use crate::stream;
fn main() {
	let s = stream::Create_stream("ETHBTC", "BNBETH", "ETHUSD");
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
