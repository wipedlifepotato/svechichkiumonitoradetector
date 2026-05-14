use crate::stream::PricePair;
use std::collections::HashMap;
use crate::strategy::ArbitrageStrategy;

pub struct TriangleArbitrage {
    pub p1: String,
    pub p2: String,
    pub p3: String,
    pub threshold: f64,
    pub initial_asset: String,
}

// by gemini/gpt
impl TriangleArbitrage {
    fn convert(amount: f64, from: &str, pair: &str, p_data: &PricePair) -> Option<(f64, String)> {
        let bid = *p_data.bid_price.first()?;
        let ask = *p_data.ask_price.first()?;
        let fee = 0.999; // 0.1%

        if pair.starts_with(from) {
            let next_asset = pair.replace(from, ""); 
            let result = (amount * bid) * fee;
            Some((result, next_asset))
        } else if pair.ends_with(from) {
            let next_asset = pair.replace(from, "");
            let result = (amount / ask) * fee;
            Some((result, next_asset))
        } else {
            None
        }
    }
}

impl ArbitrageStrategy for TriangleArbitrage {
    fn analyze(&self, data: &HashMap<String, Vec<PricePair>>) -> Option<String> {
        let v1 = data.get(&self.p1)?.last()?;
        let v2 = data.get(&self.p2)?.last()?;
        let v3 = data.get(&self.p3)?.last()?;

        let start_amount = 100.0;
        let initial_asset = &self.initial_asset; // Берем ссылку напрямую

        let (res1, asset1) = Self::convert(start_amount, initial_asset, &self.p1, v1)?;
        
        let (res2, asset2) = Self::convert(res1, &asset1, &self.p2, v2)?;
        
        let (res3, asset3) = Self::convert(res2, &asset2, &self.p3, v3)?;

        if asset3 != *initial_asset {
            return None; 
        }

        let profit_percent = ((res3 - start_amount) / start_amount) * 100.0;

        if profit_percent > self.threshold {
            Some(format!(
                "💎 Profit: {:+.4}% | {} -> {} -> {} | {} -> {:.4} {}",
                profit_percent, 
                self.p1, self.p2, self.p3, 
                start_amount, res3, asset3
            ))
        } else {
            None
        }
    }
}
