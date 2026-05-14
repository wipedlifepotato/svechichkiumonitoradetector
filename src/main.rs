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
    //dbg!(&pair);
    if let Some(history) = lock.get(&pair) {
	//	dbg!(&history);
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


fn create_strategy(cfg: &config::Config) -> Box<dyn ArbitrageStrategy> {
    match cfg.mode {
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
    }
}

async fn run_lua_scripts(
    app_data: Arc<Mutex<HashMap<String, Vec<PricePair>>>>,
    pairs: Vec<String>,
    strategy: Arc<Box<dyn ArbitrageStrategy>>
) {
    if let Ok(entries) = std::fs::read_dir("./scripts") {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("lua") {
                let d = Arc::clone(&app_data);
                let p = pairs.clone();
                let s = Arc::clone(&strategy);
                let script_path = path.clone();

                tokio::task::spawn_blocking(move || {
                    if let Ok(lua) = crate::lua_logic::init_lua(d, p, s) {
                        let _ = lua.load(script_path).exec();
                    }
                });
            }
        }
    }
}


// TODO: fix multiple pairs name ?
async fn perform_ai_analysis(
    data: Arc<Mutex<HashMap<String, Vec<PricePair>>>>,
    cfg: config::Config,
    strategy: Arc<Box<dyn ArbitrageStrategy>>
) {
    let ai = crate::ai::GeminiClient::new();
    
    let mut snapshots = Vec::new();

    {
        if let Ok(locked_data) = data.lock() {
            for pair_name in &cfg.pairs {
                if let Some(history) = locked_data.get(pair_name) {
                    if history.len() >= 90 {
                        snapshots.push((pair_name.clone(), history.clone()));
                    }
                }
            }
        }
    }

    for (pair_name, history) in snapshots {
        if let Ok(d_json) = serde_json::to_string(&history) {
            if let Ok(opinion) = ai.analyze(&pair_name, &d_json).await {
                if !opinion.is_empty() {
                    strategy.alert(&opinion).await;
                }
            }
        }
    }
}

#[tokio::main]
async fn main() {
	dotenvy::dotenv().ok();
    let cfg = config::Config::load();
    let s = stream::create_stream(cfg.pairs.clone());
    
    let strategy = Arc::new(create_strategy(&cfg));

    let server_data = Arc::clone(&s.values);
    let server_port = cfg.port;
    tokio::spawn(async move {
        let app = Router::new()
            .route("/chart/:pair", get(handle_get_chart))
            .route("/history/:pair", get(handle_json_history))
            .layer(Extension(server_data));

        let addr = SocketAddr::from(([127, 0, 0, 1], server_port));
        let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
        println!("🚀 HTTP Server listening on http://{}", addr);
        axum::serve(listener, app).await.unwrap();
    });

    if env::var("LUA_ENABLED").map(|v| v.to_uppercase() == "TRUE").unwrap_or(true) {
        let app_data = Arc::clone(&s.values);
        let pairs = cfg.pairs.clone();
        let strat = Arc::clone(&strategy);
        
        tokio::task::spawn(async move {
            run_lua_scripts(app_data, pairs, strat).await;
        });
    }

    let mut ai_timer = std::time::Instant::now();
    
    loop {
        {
            if let Ok(data) = s.values.try_lock() { 
                if let Some(msg) = strategy.analyze(&data) {
                    let s_clone = Arc::clone(&strategy);
                    tokio::spawn(async move { s_clone.alert(&msg).await });
                }
            }
        }
		//println!("{}",env::var("GEMINI_ENABLED").unwrap_or("TRUE".to_string()).to_uppercase() == "TRUE");
		//println!("{}", ai_timer.elapsed().as_secs() );
        if env::var("GEMINI_ENABLED").unwrap_or("TRUE".to_string()).to_uppercase() == "TRUE" && ai_timer.elapsed().as_secs() >= 60 {
			println!("Call gemini");
            let data_clone = Arc::clone(&s.values);
            let cfg_clone = cfg.clone();
            let strat_clone = Arc::clone(&strategy);
            
            tokio::spawn(async move {
                perform_ai_analysis(data_clone, cfg_clone, strat_clone).await;
            });
            ai_timer = std::time::Instant::now();
        }

        tokio::time::sleep(Duration::from_millis(100)).await;
    }
}
