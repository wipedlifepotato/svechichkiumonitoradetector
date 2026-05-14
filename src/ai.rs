use serde_json::json;
use std::env;

pub struct GeminiClient {
    api_key: String,
    client: reqwest::Client,
}

impl GeminiClient {
pub fn new() -> Self {
        let api_key = std::env::var("GEMINI_API_KEY").expect("GEMINI_API_KEY not found");
        
        let mut client_builder = reqwest::Client::builder();

        if let Ok(proxy_url) = std::env::var("PROXY_URL") {
            let proxy = reqwest::Proxy::all(proxy_url).expect("Invalid Proxy URL");
            client_builder = client_builder.proxy(proxy);
        }

        Self {
            api_key,
            client: client_builder.build().expect("Failed to build reqwest client"),
        }
    }

    pub async fn analyze(&self, pair: &str, data: &str) -> Result<String, Box<dyn std::error::Error>> {
		let model_name = "gemini-3.1-flash-lite"; 
		let url = format!(
			"https://generativelanguage.googleapis.com/v1beta/models/{}:generateContent?key={}",
			model_name,
			self.api_key
		);
		let prompt = format!(
			"You are a crypto trading expert. Analyze the market data for {pair}. \
			Recent price and volume changes: {data}. \
			Is this organic growth or market manipulation (pump and dump)? \
			Reply concisely in one sentence. Start with 'REAL' or 'MANIPULATION', \
			then briefly state if the price is likely to 'CONTINUE' or 'REVERT'.",
			pair = pair,
			data = data
		);
        let body = json!({
            "contents": [{
                "parts": [{ "text": prompt }]
            }]
        });
		let resp = self.client.post(&url)
					.json(&body)
					.send()
					.await?;

				let status = resp.status();
				let json_resp: serde_json::Value = resp.json().await?;
				
				if !status.is_success() {
					eprintln!("Gemini API Error: {} - {}", status, json_resp);
					return Err(format!("API returned error: {}", status).into());
				}

				let ai_text = json_resp["candidates"][0]["content"]["parts"][0]["text"]
					.as_str()
					.map(|s| s.to_string())
					.ok_or_else(|| {
						eprintln!("Unexpected JSON structure: {:?}", json_resp);
						"Не удалось распарсить ответ от ИИ"
					})?;

				Ok(ai_text)
		  }
}
