use crate::stream::PricePair;
use std::collections::HashMap;
use crate::strategy::ArbitrageStrategy;
use crate::config::Mode;

pub struct PriceChangeDetector {
    pub pairs: Vec<String>,
    pub mode: Mode,
    pub threshold: f64,
    pub target_price: Option<f64>,
}
//by gemini
impl ArbitrageStrategy for PriceChangeDetector {
    fn analyze(&self, data: &HashMap<String, Vec<PricePair>>) -> Option<String> {
        let mut reports = Vec::new();

        for pair_name in &self.pairs {
            if let Some(history) = data.get(pair_name) {
                if history.len() < 2 { continue; }

				let mut iter = history.iter().rev(); 
				let current = iter.next()?;         
				let previous = iter.next()?; 
				
                if current.timestamp <= previous.timestamp {
                    continue;
                }

                let current_price = *current.bid_price.first()?;
                let prev_price = *previous.bid_price.first()?;

                if (current_price - prev_price).abs() < f64::EPSILON {
                    continue;
                }

                let change = ((current_price - prev_price) / prev_price) * 100.0;

                match self.mode {
                    Mode::Pump if change >= self.threshold => {
                        reports.push(format!(
                            "🚀 PUMP: {} | {:.2}% | {:.4} -> {:.4} | ts: {}", 
                            pair_name, change, prev_price, current_price, current.timestamp
                        ));
                    },
                    Mode::Dump if change <= self.threshold => {
                        reports.push(format!(
                            "📉 DUMP: {} | {:.2}% | {:.4} -> {:.4} | ts: {}", 
                            pair_name, change, prev_price, current_price, current.timestamp
                        ));
                    },
                    _ => {}
                }

                if let Some(target) = self.target_price {
                    if (prev_price < target && current_price >= target) || 
                       (prev_price > target && current_price <= target) {
                        reports.push(format!("🎯 TARGET REACHED: {} @ {:.4}", pair_name, current_price));
                    }
                }
            }
        }

        if reports.is_empty() { None } else { Some(reports.join("\n")) }
    }
}
