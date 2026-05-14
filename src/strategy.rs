//? strategys
/*
 * 	Interface, get pars data and give option String, or none
 * ?*/
use std::collections::HashMap;
use crate::stream::PricePair;
//use serde_json;

use crate::notify::{DesktopNotifier, Notifier};
use crate::notify::WebhookNotifier;

pub trait ArbitrageStrategy {
    fn analyze(&self, pairs: &HashMap<String, Vec<PricePair>>) -> Option<String>;  
    fn notify(&self, message: &str){ 
		// todo: tg
		todo!("not implemented: {}", message);
	}  
    fn alert(&self, message: &str) {
		//let w = WebhookNotifier { url:"".to_string() };
		let d = DesktopNotifier {};
		//w.send(message);
		d.send(message);
        println!("[ALERT]: {}", message);
        
        
    }
}


