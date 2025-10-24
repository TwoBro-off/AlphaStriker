use serde::{Deserialize, Serialize};
use tokio::time::{sleep, Duration};
use tracing::error;
use uuid::Uuid;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct StrategySnapshot {
    pub buy_amount_sol: f64,
    pub sell_multiplier: f64,
    pub trailing_stop_percent: f64,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CreatorRecord {
    pub creator: String,
    pub first_seen: i64,
    pub last_win: i64,
    pub hits: u32,
}
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TradeRecord {
    pub id: String,
    pub token_address: String,
    pub buy_timestamp: i64,
    pub buy_price: f64,
    pub amount_sol: f64,
    pub amount_tokens: f64,
    pub sell_timestamp: Option<i64>,
    pub sell_price: Option<f64>,
    pub pnl_sol: Option<f64>,
    pub pnl_percent: Option<f64>,
    pub strategy_snapshot: Option<StrategySnapshot>,
    pub run_mode: String, // Ajouté pour suivre si c'était un trade réel ou démo
}

#[derive(Clone)]
pub struct Database {
    db: sled::Db,
}

impl Database {
    pub fn new(path: &str) -> Result<Self, sled::Error> {
        let db = sled::open(path)?;
        Ok(Database { db })
    }

    pub fn save_trade(&self, trade: &TradeRecord) -> Result<(), sled::Error> {
        let key = trade.id.as_bytes();
        let value = match bincode::serialize(trade) {
            Ok(v) => v,
            Err(e) => return Err(sled::Error::Unsupported(e.to_string())),
        };
        self.db.insert(key, value)?;
        Ok(())
    }

    pub fn get_trade(&self, id: &str) -> Result<Option<TradeRecord>, sled::Error> {
        let result = self.db.get(id)?;
        match result {
            Some(ivec) => {
                let trade: TradeRecord = match bincode::deserialize(&ivec) {
                    Ok(t) => t,
                    Err(e) => return Err(sled::Error::Unsupported(e.to_string())),
                };
                Ok(Some(trade))
            }
            None => Ok(None),
        }
    }

    pub fn get_all_trades(&self) -> Result<Vec<TradeRecord>, sled::Error> {
        let mut trades = Vec::new();
        for item in self.db.iter() {
            let (_, value) = item?;
            let trade: TradeRecord = match bincode::deserialize(&value) {
                Ok(t) => t,
                Err(e) => return Err(sled::Error::Unsupported(e.to_string())),
            };
            trades.push(trade);
        }
        Ok(trades)
    }

    pub fn get_recent_trades(&self, limit: usize) -> Result<Vec<TradeRecord>, sled::Error> {
        let mut trades = self.get_all_trades()?;
        trades.sort_by(|a, b| b.buy_timestamp.cmp(&a.buy_timestamp));
        trades.truncate(limit);
        Ok(trades)
    }

    pub async fn save_trade_with_retry(&self, trade: TradeRecord) {
        let mut attempts = 0;
        while attempts < 3 {
            match self.save_trade(&trade) {
                Ok(_) => return,
                Err(e) => {
                    error!("[DB] Tentative {}/3: Échec de la sauvegarde du trade {}: {}", attempts + 1, trade.id, e);
                    attempts += 1;
                    sleep(Duration::from_millis(100)).await;
                }
            }
        }
        error!("[DB] Échec final de la sauvegarde du trade {} après 3 tentatives.", trade.id);
    }
}