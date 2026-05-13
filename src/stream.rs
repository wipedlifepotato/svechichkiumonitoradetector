//use binance::model::*;
use binance::websockets::*;
use std::sync::atomic::{AtomicBool};
use std::thread;
use std::sync::{Arc, Mutex}; 
use std::collections::HashMap;
use std::time::SystemTime;

#[derive(Debug, Clone)]
pub struct PricePair {
    pub bid_price: Vec<f64>,
    pub ask_price: Vec<f64>,
    pub qty_bid: Vec<f64>,
    pub qty_ask: Vec<f64>,
    pub timestamp: u64,
}

pub struct Stream {
    pub keep_running: Arc<AtomicBool>,
    pub handle: thread::JoinHandle<()>,
    pub values: Arc<Mutex<HashMap<String, Vec<PricePair>>>>,
}

pub fn create_stream_3(pair1: &str, pair2: &str, pair3: &str) -> Stream {
    let pairs = vec![pair1.to_string(), pair2.to_string(), pair3.to_string()];
    
    let endpoints: Vec<String> = pairs
        .iter()
        .map(|symbol| format!("{}@depth@100ms", symbol.to_lowercase()))
        .collect();

    let keep_running = Arc::new(AtomicBool::new(true));

    let mut initial_map = HashMap::new();
    for p in &pairs {
        initial_map.insert(p.to_uppercase(), Vec::new());
    }
    
    let hmap = Arc::new(Mutex::new(initial_map));
    
    let kr_clone = Arc::clone(&keep_running);
    let hmap_clone = Arc::clone(&hmap);

    let handle = thread::spawn(move || {
        let mut web_socket = WebSockets::new(move |event: WebsocketEvent| {
            if let WebsocketEvent::DepthOrderBook(depth_order_book) = event {
                let symbol = depth_order_book.symbol.to_uppercase();
                
                let mut bids = Vec::new();
                let mut bids_qty = Vec::new();
                let mut asks = Vec::new();
                let mut asks_qty = Vec::new();

                for bid in depth_order_book.bids {
                    bids.push(bid.price);
                    bids_qty.push(bid.qty);
                }
                for ask in depth_order_book.asks {
                    asks.push(ask.price);
                    asks_qty.push(ask.qty);
                }

                let new_data = PricePair {
                    bid_price: bids,
                    ask_price: asks,
                    qty_bid: bids_qty,
                    qty_ask: asks_qty,
                    timestamp: SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).expect("Can't get timestamp").as_secs(),
                };

                if let Ok(mut map) = hmap_clone.lock() {
                    if let Some(vec) = map.get_mut(&symbol) {
                        vec.push(new_data);
                    }
                }
            }
            Ok(())
        });

        web_socket.connect_multiple_streams(&endpoints).expect("Failed to connect");
        
        if let Err(e) = web_socket.event_loop(&kr_clone) {
            println!("Error in event loop: {:?}", e);
        }
        
        web_socket.disconnect().unwrap();
    });

    Stream {
        keep_running,
        handle,
        values: hmap,
    }
}
