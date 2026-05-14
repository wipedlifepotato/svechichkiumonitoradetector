mod stream;
mod config;
mod logic_arbitrage;
mod logic_pricechange;
mod strategy;
mod notify;
mod charts;
mod ai;
mod lua_logic;

use std::env;
use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use std::time::Duration;
use crate::strategy::ArbitrageStrategy;
use crate::config::Mode;
use crate::stream::PricePair;
use axum::Extension;
use crate::logic_arbitrage::TriangleArbitrage; 
use crate::logic_pricechange::PriceChangeDetector;
use axum::{
    routing::get, 
    Router, 
    extract::Path, 
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
        if history.is_empty() { return "No data".into_response(); }
        let png = crate::charts::generate_chart_png(&pair, history);
        return ([(header::CONTENT_TYPE, "image/png")], png).into_response();
    }
    (axum::http::StatusCode::NOT_FOUND, "Not found").into_response()
}

async fn handle_json_history(
    Path(pair): Path<String>,
    Extension(data): Extension<Arc<Mutex<HashMap<String, Vec<PricePair>>>>>
) -> impl IntoResponse {
    let pair = pair.to_uppercase();
    let lock = data.lock().unwrap();
    if let Some(history) = lock.get(&pair) {
        let data = serde_json::to_string(&history).unwrap();
        return ([(header::CONTENT_TYPE, "text/json")], data).into_response();
    }
    (axum::http::StatusCode::NOT_FOUND, "Not found").into_response()
}

#[tokio::main]
async fn main() {
    let cfg = config::Config::load();
    let pairs_to_stream = cfg.pairs.clone();
    let s = stream::create_stream(pairs_to_stream);

    let raw_strategy: Box<dyn ArbitrageStrategy> = match cfg.mode {
        Mode::Triangle => {
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
	let strategy = Arc::new(raw_strategy);

	let app_data_for_lua = Arc::clone(&s.values);
	let pairs_for_lua = cfg.pairs.clone();
	let strategy_for_lua = Arc::clone(&strategy);


	if env::var("LUA_ENABLED").unwrap_or("TRUE".to_string()).to_uppercase() == "TRUE" {
		if let Ok(entries) = std::fs::read_dir("./scripts") {
			for entry in entries.flatten() {
				let path = entry.path();
				
				if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("lua") {
					
					let app_data = Arc::clone(&s.values);
					let pairs = cfg.pairs.clone();
					let strategy = Arc::clone(&strategy);
					let script_path = path.clone();

					tokio::task::spawn_blocking(move || {
						println!("Launching Lua instance for: {:?}", script_path);
						
						let lua_res = crate::lua_logic::init_lua(app_data, pairs, strategy);

						match lua_res {
							Ok(lua) => {
								if let Err(e) = lua.load(script_path.clone()).exec() {
									println!("Error in script {:?}: {:?}", script_path, e);
								}
							}
							Err(e) => println!("Failed to init Lua for {:?}: {:?}", script_path, e),
						}
					});
				}
			}
		}
	}
    println!("Detector running | Mode: {:?} | Pairs: {}", cfg.mode, cfg.pairs.len());

    let server_data = Arc::clone(&s.values);
    let server_port = cfg.port;
    tokio::spawn(async move {
        let app = Router::new()
            .route("/chart/:pair", get(handle_get_chart))
            .route("/history/:pair", get(handle_json_history))
            .layer(axum::Extension(server_data));

        let addr = SocketAddr::from(([127, 0, 0, 1], server_port));
        let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
        println!("HTTP Server listening on http://{}", addr);
        axum::serve(listener, app).await.unwrap();
    });

    let mut now_time = std::time::Instant::now();
    let mut first_launch = true;

    loop {
        if let Ok(data) = s.values.lock() {
            if let Some(msg) = strategy.analyze(&data) {
                strategy.alert(&msg).await;
            }
            if env::var("GEMINI_ENABLED").unwrap_or_default().to_uppercase() == "TRUE" {
                if first_launch || now_time.elapsed().as_secs() >= 60 {
                    let ai = crate::ai::GeminiClient::new();
                    for pair_name in &cfg.pairs {
                        if let Some(history) = data.get(pair_name) {
                            if history.len() >= 90 {
                                let d = serde_json::to_string(&history).unwrap();
                                if let Ok(opinion) = ai.analyze(pair_name, &d).await {
                                    if !opinion.is_empty() { strategy.alert(&opinion).await; }
                                }
                            }
                        }
                    }
                    now_time = std::time::Instant::now();
                    first_launch = false;
                }
            }
        }
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
}
