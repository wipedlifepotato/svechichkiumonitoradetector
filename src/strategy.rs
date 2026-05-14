use async_trait::async_trait;
use std::collections::HashMap;
use std::env;
use crate::notify::{Notifier, TGNotifier, DesktopNotifier}; 
use crate::PricePair;
use dotenvy::dotenv;

#[async_trait]
pub trait ArbitrageStrategy: Send + Sync {
    fn analyze(&self, pairs: &HashMap<String, Vec<PricePair>>) -> Option<String>;

    async fn alert(&self, message: &str) {
        println!("[ALERT]: {}", message);
		dotenv().ok();
        let desktop_enabled = env::var("ALERT_DESKTOP_ENABLED").unwrap_or("false".to_string());
        if desktop_enabled.to_uppercase() == "TRUE" {
            let n = DesktopNotifier;
            let _ = n.send(message, None).await;
        }

        let tg_enabled = env::var("ALERT_TELEGRAM_ENABLED").unwrap_or("false".to_string());
        //dbg!(&tg_enabled);
        if tg_enabled.to_uppercase() == "TRUE" {
            let cfg = crate::config::Config::load();

            let token = env::var("BOT_TG_FATHER_KEY").unwrap_or_default();

            let chat_id = env::var("BOT_TG_USERID").unwrap_or_default();
            let charts_base_url = env::var("CHARTS_URL").unwrap_or(format!("http://127.0.0.1:{}/charts", cfg.port).to_string());

            let n = TGNotifier::new(&token, &chat_id);
			let charts_base_url = env::var("CHARTS_URL").unwrap_or("http://127.0.0.1:3000".to_string());
			for pair_name in cfg.pairs {
			let chart_url = format!("{}/chart/{}", charts_base_url, pair_name.to_uppercase());
			
			println!("DEBUG: trying load a graph from {}", chart_url);

			let mut image_data = None;
			match reqwest::get(&chart_url).await {
				Ok(resp) => {
					if resp.status().is_success() {
						match resp.bytes().await {
							Ok(bytes) => {
								image_data = Some(bytes.to_vec());
								println!("DEBUG: Successfully downloaded bytes: {}", image_data.as_ref().unwrap().len());
							}
							Err(e) => println!("DEBUG: Err reading bytes: {}", e),
						}
					} else {
						println!("DEBUG: Server a give an error: {}", resp.status());
					}
				}
				Err(e) => println!("DEBUG: Can't reach a server of graph: {}", e),
			}

			if let Err(e) = n.send(message, image_data).await {
				eprintln!("Ошибка при отправке в TG: {}", e);
		 }
	 }
 }
}
}
