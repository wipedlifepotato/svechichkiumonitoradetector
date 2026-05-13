pub trait Notifier {
    fn send(&self, msg: &str);
}

/// desktop
use notify_rust::Notification;

pub struct DesktopNotifier;
impl Notifier for DesktopNotifier {
    fn send(&self, msg: &str) {
		 match(Notification::new()
		.summary("Huita Detected")
		.body(msg)
		//.icon("firefox")
		.show()) {
				Ok(_) => {
					
				},
				Err(_) => {
					eprintln!("Can't send notification");
					eprintln!("{}",msg);
				}
		}
        //todo!("not implemented");
    }
}

/// webhook
pub struct WebhookNotifier {
    pub url: String,
}

impl Notifier for WebhookNotifier {
    fn send(&self, message: &str) {
        todo!("not implemented");
    }
}

//end notifer
