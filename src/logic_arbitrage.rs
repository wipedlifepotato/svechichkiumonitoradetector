use crate::stream::PricePair;
use std::collections::HashMap;
use crate::strategy::ArbitrageStrategy;

pub struct TriangleArbitrage {
    pub p1: String,
    pub p2: String,
    pub p3: String,
    pub threshold: f64,
}

// by gemini/gpt
impl ArbitrageStrategy for TriangleArbitrage {
    fn analyze(&self, data: &HashMap<String, Vec<PricePair>>) -> Option<String> {
        // 1. Извлекаем последние данные стакана для каждой пары
        let v1 = data.get(&self.p1)?.last()?;
        let v2 = data.get(&self.p2)?.last()?;
        let v3 = data.get(&self.p3)?.last()?;

        // 2. Берем лучшие цены покупки (Ask) и продажи (Bid)
        let ask1 = *v1.ask_price.first()?; 
        let ask2 = *v2.ask_price.first()?; 
        let bid3 = *v3.bid_price.first()?; 

        // Защита от нулевых цен (бывает при лагах API)
        if ask1 <= 0.0 || bid3 <= 0.0 { return None; }

        // 3. Расчет цепочки: USDT -> BTC -> ETH -> USDT
        let start_amount = 100.0;
        let fee = 0.999; // Комиссия 0.1% на каждом шаге

        // Шаг 1: USDT -> BTC (Покупка по Ask)
        let btc_amount = (start_amount / ask1) * fee;
        
        // Шаг 2: BTC -> ETH (Покупка по Ask)
        let eth_amount = (btc_amount / ask2) * fee;
        
        // Шаг 3: ETH -> USDT (Продажа по Bid)
        let final_usdt = (eth_amount * bid3) * fee;

        // 4. Считаем чистый профит в процентах
        let profit_percent = ((final_usdt - start_amount) / start_amount) * 100.0;

        if profit_percent > self.threshold {
            Some(format!(
                "💎 ПРОФИТ: {:+.4}% | {} -> {} -> {} | {} -> {:.2} USDT",
                profit_percent, self.p1, self.p2, self.p3, start_amount, final_usdt
            ))
        } else {
            None
        }
    }
}
