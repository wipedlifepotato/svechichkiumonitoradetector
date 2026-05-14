use async_trait::async_trait;
use reqwest::multipart;
use serde_json::json;
use notify_rust::Notification;

#[async_trait]
pub trait Notifier: Send + Sync { 
    async fn send(&self, msg: &str, photo: Option<Vec<u8>>) -> Result<(), reqwest::Error>;
}

pub struct TGNotifier {
    token: String,
    chat_id: String,
}

impl TGNotifier {
    pub fn new(token: &str, chat_id: &str) -> Self {
        Self {
            token: token.to_string(),
            chat_id: chat_id.to_string(),
        }
    }

    async fn send_raw(&self, message: &str, photo_bytes: Option<Vec<u8>>) -> Result<(), reqwest::Error> {
        let client = reqwest::Client::new();
        
        let text_url = format!("https://api.telegram.org/bot{}/sendMessage", self.token);
        client.post(&text_url)
            .json(&json!({
                "chat_id": self.chat_id,
                "text": message,
                "parse_mode": "Markdown"
            }))
            .send()
            .await?;

        if let Some(photo) = photo_bytes {
            let photo_url = format!("https://api.telegram.org/bot{}/sendPhoto", self.token);
            let part = multipart::Part::bytes(photo).file_name("chart.png");
            let form = multipart::Form::new()
                .text("chat_id", self.chat_id.clone())
                .part("photo", part);

            client.post(&photo_url).multipart(form).send().await?;
        }

        Ok(())
    }
}

#[async_trait]
impl Notifier for TGNotifier {
    async fn send(&self, message: &str, photo: Option<Vec<u8>>) -> Result<(), reqwest::Error> {
        self.send_raw(message, photo).await
    }
}

/// desktop
pub struct DesktopNotifier;

#[async_trait]
impl Notifier for DesktopNotifier {
    async fn send(&self, msg: &str, _photo: Option<Vec<u8>>) -> Result<(), reqwest::Error> {
        if let Err(e) = Notification::new()
            .summary("Huita Detected")
            .body(msg)
            .show() 
        {
            eprintln!("Can't send desktop notification: {}", e);
            eprintln!("{}", msg);
        }
        Ok(()) 
    }
}

/// webhook
pub struct WebhookNotifier {
    pub url: String,
}

#[async_trait]
impl Notifier for WebhookNotifier {
    async fn send(&self, message: &str, _photo: Option<Vec<u8>>) -> Result<(), reqwest::Error> {
        println!("Webhook (not implemented) to {}: {}", self.url, message);
        Ok(())
    }
}
