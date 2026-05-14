mod stream;
mod config;
mod logic_arbitrage;
mod logic_pricechange;
mod strategy;
mod notify;
mod charts;

use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use std::time::Duration;
use crate::strategy::ArbitrageStrategy;
use crate::config::Mode;
use crate::stream::{PricePair};
use axum::Extension;
use crate::logic_arbitrage::TriangleArbitrage; 
use crate::logic_pricechange::PriceChangeDetector;

use axum::{
    routing::get, 
    Router, 
    extract::{Path}, 
    response::IntoResponse,
    http::header,
};
use std::net::SocketAddr;

async fn handle_get_chart(
    Path(pair): Path<String>,
    Extension(data): Extension<Arc<Mutex<HashMap<String, Vec<PricePair>>>>>
) -> impl IntoResponse {
    let pair = pair.to_uppercase();
    let lock = data.lock().unwrap();
    
    if let Some(history) = lock.get(&pair) {
        if history.is_empty() {
            return "No data points collected yet".into_response();
        }
        let png = crate::charts::generate_chart_png(&pair, history);
        return (
            [(header::CONTENT_TYPE, "image/png")],
            png
        ).into_response();
    }

    (axum::http::StatusCode::NOT_FOUND, "Pair not found").into_response()
}

async fn handle_json_history(
    Path(pair): Path<String>,
    Extension(data): Extension<Arc<Mutex<HashMap<String, Vec<PricePair>>>>>
) -> impl IntoResponse {
	
	let pair = pair.to_uppercase();
    let lock = data.lock().unwrap();
    
    if let Some(history) = lock.get(&pair) {
        if history.is_empty() {
            return "No data points collected yet".into_response();
        }
        let data = serde_json::to_string(&history).unwrap();
        return (
            [(header::CONTENT_TYPE, "text/json")],
            data
        ).into_response();
	}
	(axum::http::StatusCode::NOT_FOUND, "Pair not found").into_response()
}

#[tokio::main]
async fn main() {
    let cfg = config::Config::load();
    let pairs_to_stream = cfg.pairs.clone();

    let s = stream::create_stream(pairs_to_stream);

    let strategy: Box<dyn ArbitrageStrategy> = match cfg.mode {
        Mode::Triangle => {
            if cfg.pairs.len() < 3 {
                panic!("Triangle mode needs at least 3 pairs!");
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

    println!("Detector running | Mode: {:?} | Pairs: {}", cfg.mode, cfg.pairs.len());
    println!("Charts at http://127.0.0.1:{}/chart/<PAIR>", cfg.port);

    let app_data = Arc::clone(&s.values); 

    tokio::spawn(async move {
        let app = Router::new()
            .route("/chart/:pair", get(handle_get_chart))
            .route("/history/:pair", get(handle_json_history))
            .layer(axum::Extension(app_data));

        let addr = SocketAddr::from(([127, 0, 0, 1], cfg.port));
        let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
        
        println!("HTTP Server listening on http://{}", addr);
        axum::serve(listener, app).await.unwrap();
    });

    loop {
        if let Ok(data) = s.values.lock() {
            if let Some(msg) = strategy.analyze(&data) {
//                println!("{}", msg); 
				  strategy.alert(&msg);
            }
        }
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
}
