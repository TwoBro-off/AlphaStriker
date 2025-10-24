use dashmap::DashMap;
use std::sync::Arc;
use crate::db::StrategySnapshot;
use std::env;
use tracing::{info, warn};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize)]
pub struct Position {
    pub trade_id: String,
    pub token_address: String,
    pub buy_price: f64,
    pub amount_sol: f64,
    pub amount_tokens: f64,
    pub buy_timestamp: i64,
    pub highest_price: f64,
    pub strategy_snapshot: StrategySnapshot,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SaleDecision {
    Hold,
    SellTakeProfit,
    SellTrailingStop,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StrategyParams {
    pub buy_amount_sol: f64,
    pub sell_multiplier: f64,
    pub trailing_stop_percent: f64,
}

#[derive(Clone)]
pub struct TradingCore {
    positions: Arc<DashMap<String, Position>>,
    pub buy_amount_sol: f64,
    pub sell_multiplier: f64,
    pub trailing_stop_percent: f64,
    pub max_trade_duration_secs: u64,
}

impl TradingCore {
    pub fn new(buy_amount_sol: f64, sell_multiplier: f64, trailing_stop_percent: f64) -> Self {
        TradingCore {
            positions: Arc::new(DashMap::new()),
            buy_amount_sol,
            sell_multiplier,
            trailing_stop_percent,
            max_trade_duration_secs: env::var("MAX_TRADE_DURATION_SECS")
                .unwrap_or_else(|_| "14400".to_string())
                .parse()
                .unwrap_or(14400),
        }
    }

    pub fn set_strategy_params(&mut self, buy_amount_sol: f64, sell_multiplier: f64, trailing_stop_percent: f64) {
        self.buy_amount_sol = buy_amount_sol;
        self.sell_multiplier = sell_multiplier;
        self.trailing_stop_percent = trailing_stop_percent;
        info!(
            "[Trading Core] Paramètres de stratégie mis à jour: buy_amount={:.4} SOL, sell_mult={:.2}x, trail_stop={:.2}%",
            buy_amount_sol, sell_multiplier, trailing_stop_percent * 100.0
        );
    }

    pub fn get_strategy_params(&self) -> StrategyParams {
        StrategyParams {
            buy_amount_sol: self.buy_amount_sol, // Ajout pour la complétude
            sell_multiplier: self.sell_multiplier,
            trailing_stop_percent: self.trailing_stop_percent,
        }
    }

    pub fn add_position(&self, trade_id: String, token_address: String, buy_price: f64, amount_sol: f64, amount_tokens: f64, buy_timestamp: i64, strategy_snapshot: StrategySnapshot) {
        let position = Position { 
            trade_id, token_address: token_address.clone(), buy_price, amount_sol, amount_tokens, buy_timestamp, strategy_snapshot,
            highest_price: buy_price,
        };
        self.positions.insert(token_address, position);
    }

    pub fn remove_position(&self, token_address: &str) -> Option<Position> {
        self.positions.remove(token_address).map(|(_k, v)| v)
    }

    pub fn get_position(&self, token_address: &str) -> Option<Position> {
        self.positions.get(token_address).map(|p| p.value().clone())
    }

    pub fn evaluate_for_sale(&self, token_address: &str, current_price: f64) -> SaleDecision {
        if let Some(mut position) = self.positions.get_mut(token_address) {
            let buy_price = position.buy_price;

            if buy_price <= 0.0 { // Should not happen, but as a safeguard
                warn!("[Trading Core] Position {} a un prix d'achat nul ou négatif. Impossible d'évaluer.", token_address);
                return SaleDecision::Hold;
            }

            let profit_multiplier = current_price / buy_price;
            if profit_multiplier >= self.sell_multiplier {
                return SaleDecision::SellTakeProfit;
            }

            if current_price > position.highest_price {
                position.highest_price = current_price;
            }

            let stop_price = position.highest_price * (1.0 - self.trailing_stop_percent);
            if current_price < stop_price {
                return SaleDecision::SellTrailingStop;
            }
        }
        SaleDecision::Hold
    }

    pub fn get_all_positions_internal(&self) -> Vec<(String, Position)> {
        self.positions.iter().map(|item| (item.key().clone(), item.value().clone())).collect()
    }
}