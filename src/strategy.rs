//? strategys
/*
 * 	Interface, get pars data and give option String, or none
 * ?*/
use std::collections::HashMap;
use crate::stream::PricePair;
pub trait ArbitrageStrategy {
    fn analyze(&self, pairs: &HashMap<String, Vec<PricePair>>) -> Option<String>;  
    fn notify(&self, message: &str){ 
		// todo: tg
		todo!("not implemented");
	}  
    fn alert(&self, message: &str) {
        println!("[ALERT]: {}", message);
    }
}


