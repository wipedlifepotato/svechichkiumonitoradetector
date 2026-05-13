use crate::stream::PricePair;
use std::collections::HashMap;
use crate::stream::strategy::*;

pub struct TriangleArbitrage {
    pub p1: String, // BTCUSDT
    pub p2: String, // ETHBTC
    pub p3: String, // ETHUSDT
    pub threshold: f64, // Порог профита (например, 0.001 для 0.1%)
}

// by gpt
impl ArbitrageStrategy for TriangleArbitrage {
    fn analyze(&self, data: &HashMap<String, Vec<PricePair>>) -> Option<String> {
        // Получаем последние значения (Option handling)
        let v1 = data.get(&self.p1)?.last()?;
        let v2 = data.get(&self.p2)?.last()?;
        let v3 = data.get(&self.p3)?.last()?;

        // Цены: берём первый уровень стакана как пример
        // Интерпретация: для пары BASE/QUOTE
        // - ask_price.first() — цена, по которой можно купить BASE за QUOTE
        // - bid_price.first() — цена, по которой можно продать BASE за QUOTE
        let ask1 = *v1.ask_price.first()?; // цена купить базу p1 (BASE1) за quote1
        let ask2 = *v2.ask_price.first()?; // цена купить базу p2 за quote2
        let bid3 = *v3.bid_price.first()?; // цена продать базу p3 за quote3
		dbg!(ask1,ask2,bid3);

		// start_amount в стартовой валюте (например USDT)
		let start_amount = 100.0;
		let fee = 0.999; // множитель после комиссии

		// Интерпретация: price = quote per base для пары BASE/QUOTE

		// Пример конкретной цепочки: USDT -> BTC -> ETH -> USDT
		// - p1 = BTCUSDT (BASE=BTC, QUOTE=USDT) : ask1 (USDT per 1 BTC)
		// - p2 = ETHBTC  (BASE=ETH, QUOTE=BTC)  : ask2 (BTC per 1 ETH)
		// - p3 = ETHUSDT (BASE=ETH, QUOTE=USDT) : bid3 (USDT per 1 ETH)  <-- важно: bid3 должен быть ~673.7

		// 1) USDT -> BTC : мы хотим купить BTC, т.е. сколько BTC получим за start_amount USDT?
		//    amount_btc = start_amount / ask1
		let amount_btc = (start_amount / ask1) * fee;

		// 2) BTC -> ETH : хотим купить ETH за BTC (пара ETHBTC, price = BTC per ETH).
		//    чтобы получить 1 ETH нужен ask2 BTC, значит количество ETH = amount_btc / ask2
		let amount_eth = (amount_btc / ask2) * fee;

		// 3) ETH -> USDT : продаём ETH за USDT по bid3 (USDT per ETH).
		//    final_usdt = amount_eth * bid3
		let final_usdt = amount_eth * bid3 * fee;

		// прибыль в % относительно start_amount
		let profit_percent = (final_usdt - start_amount) / start_amount * 100.0;


        let profit = final_usdt - start_amount;

        if profit_percent > self.threshold {
            Some(format!(
                "PROFIT: {:+.4}% | start {:.4} -> final {:.6} | path: {} -> {} -> {}",
                profit_percent, start_amount, final_usdt, self.p1, self.p2, self.p3
            ))
        } else {
            None
        }
    }
}
