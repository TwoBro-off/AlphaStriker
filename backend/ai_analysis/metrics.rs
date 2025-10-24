use axum::response::{IntoResponse, Response};
use serde_json::json;
use std::sync::atomic::{AtomicI64, AtomicU64, Ordering};
use std::sync::Arc;

#[derive(Debug, Default)]
pub struct Metrics {
    pub pnl_cumulated: AtomicU64, // Changed from AtomicF64
    pub total_trades_completed: AtomicU64,
    pub buy_latency_total_ms: AtomicI64,
    pub sell_latency_total_ms: AtomicI64,
    pub trade_duration_total_ms: AtomicI64,
}

impl Metrics {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn record_buy(&self, latency_ms: i64) {
        self.buy_latency_total_ms.fetch_add(latency_ms, Ordering::Relaxed);
    }

    pub fn record_sell(&self, latency_ms: i64, duration_ms: i64, pnl: f64) {
        // Atomically update the PNL by converting f64 to bits
        let pnl_bits = pnl.to_bits();
        self.pnl_cumulated.fetch_add(pnl_bits, Ordering::Relaxed);

        self.sell_latency_total_ms.fetch_add(latency_ms, Ordering::Relaxed);
        self.trade_duration_total_ms.fetch_add(duration_ms, Ordering::Relaxed);
        self.total_trades_completed.fetch_add(1, Ordering::Relaxed);
    }

    pub async fn as_json(&self) -> serde_json::Value {
        let total_trades = self.total_trades_completed.load(Ordering::Relaxed);
        let total_trades_for_avg = if total_trades == 0 { 1 } else { total_trades };
        let pnl = f64::from_bits(self.pnl_cumulated.load(Ordering::Relaxed));
        json!({
            "pnl_cumulated": pnl,
            "total_trades_completed": total_trades,
            "buy_latency_avg_ms": self.buy_latency_total_ms.load(Ordering::Relaxed) as u64 / total_trades_for_avg,
            "sell_latency_avg_ms": self.sell_latency_total_ms.load(Ordering::Relaxed) as u64 / total_trades_for_avg,
            "trade_duration_avg_ms": self.trade_duration_total_ms.load(Ordering::Relaxed) as u64 / total_trades_for_avg,
        })
    }

    pub async fn to_prometheus_format(&self) -> String {
        let total_trades = self.total_trades_completed.load(Ordering::Relaxed);
        let total_trades_for_avg = if total_trades == 0 { 1 } else { total_trades };
        let pnl = f64::from_bits(self.pnl_cumulated.load(Ordering::Relaxed));
        let buy_latency_avg = self.buy_latency_total_ms.load(Ordering::Relaxed) as f64 / total_trades_for_avg as f64;
        let sell_latency_avg = self.sell_latency_total_ms.load(Ordering::Relaxed) as f64 / total_trades_for_avg as f64;
        let trade_duration_avg = self.trade_duration_total_ms.load(Ordering::Relaxed) as f64 / total_trades_for_avg as f64;

        format!(
            "# HELP alphastriker_pnl_cumulated_sol Cumulative Profit and Loss in SOL.\n\
             # TYPE alphastriker_pnl_cumulated_sol gauge\n\
             alphastriker_pnl_cumulated_sol {}\n\
             # HELP alphastriker_trades_completed_total Total number of completed trades.\n\
             # TYPE alphastriker_trades_completed_total counter\n\
             alphastriker_trades_completed_total {}\n\
             # HELP alphastriker_buy_latency_avg_ms Average latency for buy operations in milliseconds.\n\
             # TYPE alphastriker_buy_latency_avg_ms gauge\n\
             alphastriker_buy_latency_avg_ms {}\n\
             # HELP alphastriker_sell_latency_avg_ms Average latency for sell operations in milliseconds.\n\
             # TYPE alphastriker_sell_latency_avg_ms gauge\n\
             alphastriker_sell_latency_avg_ms {}\n\
             # HELP alphastriker_trade_duration_avg_ms Average duration of a trade in milliseconds.\n\
             # TYPE alphastriker_trade_duration_avg_ms gauge\n\
             alphastriker_trade_duration_avg_ms {}\n",
            pnl, total_trades, buy_latency_avg, sell_latency_avg, trade_duration_avg
        )
    }
}

pub async fn serve_metrics(axum::extract::State(metrics): axum::extract::State<Arc<Metrics>>) -> impl IntoResponse {
    let body = metrics.to_prometheus_format().await;
    Response::builder()
        .header("Content-Type", "text/plain; version=0.0.4")
        .body(body)
        .unwrap()
}