
use binance::api::*;
use binance::model::*;
use binance::market::*;
use binance::websockets::*;
use std::sync::atomic::{AtomicBool};

use std::thread;
use std::sync::Arc;

pub struct Order_book {
	price: f64, //bigmath
	
}

pub struct Stream {
			pub keep_running: Arc<AtomicBool>,
			pub handle: thread::JoinHandle<()>,
}

/// TODO: to vec
pub fn Create_stream(pair1: &str,pair2: &str,pair3: &str) ->  Stream {
	
    let endpoints = [pair1, pair2, pair3] //["ETHBTC", "BNBETH", "ETHUSD"]
        .map(|symbol| format!("{}@depth@100ms", symbol.to_lowercase()));

    let keep_running = Arc::new(AtomicBool::new(true));
    let kr_clone = Arc::clone(&keep_running);
	let handle = thread::spawn(move || {
        let mut web_socket = WebSockets::new(|event: WebsocketEvent| {
            if let WebsocketEvent::DepthOrderBook(depth_order_book) = event {
                println!("--- Symbol: {} ---", depth_order_book.symbol);
                for bid in depth_order_book.bids {
                    println!("Bid: Price: {}, Qty: {}", bid.price, bid.qty);
                }
                for ask in depth_order_book.asks {
                    println!("Ask: Price: {}, Qty: {}", ask.price, ask.qty);
                }
            }
            Ok(())
        });

        web_socket.connect_multiple_streams(&endpoints).expect("Failed to connect");
        
        if let Err(e) = web_socket.event_loop(&keep_running) {
            println!("Error in event loop: {:?}", e);
        }
        
        web_socket.disconnect().unwrap();
    });
	Stream {
			keep_running: kr_clone,
			handle
	}
}
