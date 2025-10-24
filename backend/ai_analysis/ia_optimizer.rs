use crate::core::{BotCore, RunMode};
use crate::db::TradeRecord;
use log::{info, warn};
use std::env;
use std::time::Duration;
use tokio::time::sleep;

pub async fn run_ai_optimizer(bot_core: BotCore) {
    info!("[AI Optimizer] Démarrage du module d'optimisation.");

    if !matches!(bot_core.get_run_mode().await, RunMode::Demo) {
        warn!("[AI Optimizer] Mode REAL détecté. Le module d'optimisation est désactivé.");
        return;
    }

    let cycle_secs_str = env::var("AI_OPTIMIZER_CYCLE_SECS").unwrap_or_else(|_| "5400".to_string());
    let cycle_secs = cycle_secs_str.parse::<u64>().unwrap_or(5400);
    let optimizer_cycle_duration = Duration::from_secs(cycle_secs);

    // Attente initiale pour laisser le temps au bot de démarrer et de faire quelques trades.
    sleep(Duration::from_secs(300)).await;

    let min_trades_for_optim_str = env::var("AI_OPTIMIZER_MIN_TRADES").unwrap_or_else(|_| "20".to_string());
    let min_trades_for_optim = min_trades_for_optim_str.parse::<usize>().unwrap_or(20);

    loop {
        info!("[AI Optimizer] Lancement du cycle d'analyse et d'optimisation...");

        let trades = match bot_core.db().get_all_trades() { //NOSONAR
            Ok(trades) => trades,
            Err(e) => {
                log::error!("[AI Optimizer] Impossible de lire l'historique des trades: {}", e);
                sleep(optimizer_cycle_duration).await;
                continue;
            }
        };

        let completed_trades: Vec<&TradeRecord> = trades.iter().filter(|t| t.pnl_percent.is_some()).collect();

        if completed_trades.len() < min_trades_for_optim {
            info!("[AI Optimizer] Pas assez de trades complétés ({} sur {} requis). Prochain cycle dans {:?}.", completed_trades.len(), min_trades_for_optim, optimizer_cycle_duration);
            sleep(optimizer_cycle_duration).await;
        } else {
            let new_params = analyze_and_propose_new_params(&completed_trades, &bot_core.trading_core());
            info!("[AI Optimizer] Application des nouveaux paramètres : {:?}", new_params);
            let mut trading_core = bot_core.trading_core();
            trading_core.set_strategy_params(
                new_params.buy_amount_sol,
                new_params.sell_multiplier,
                new_params.trailing_stop_percent,
            );
            sleep(optimizer_cycle_duration).await;
        }
    }
}

#[derive(Debug)]
struct OptimizedParams {
    buy_amount_sol: f64,
    sell_multiplier: f64,
    trailing_stop_percent: f64,
}

fn analyze_and_propose_new_params(completed_trades: &[&TradeRecord], trading_core: &crate::state_manager::TradingCore) -> OptimizedParams {
    let recent_completed_trades: Vec<&&TradeRecord> = completed_trades.iter().rev().take(20).collect();

    let avg_pnl_percent = if recent_completed_trades.is_empty() {
        0.0
    } else {
        recent_completed_trades.iter().filter_map(|t| t.pnl_percent).sum::<f64>() / recent_completed_trades.len() as f64
    };

    info!("[AI Optimizer] P&L moyen sur les {} derniers trades complétés: {:.2}%", recent_completed_trades.len(), avg_pnl_percent);

    let mut new_sell_multiplier = trading_core.sell_multiplier;
    let mut new_trailing_stop = trading_core.trailing_stop_percent;
    let mut new_buy_amount_sol = trading_core.buy_amount_sol;

    let high_pnl_threshold_str = env::var("AI_OPTIMIZER_HIGH_PNL_THRESHOLD").unwrap_or_else(|_| "150.0".to_string());
    let high_pnl_threshold = high_pnl_threshold_str.parse::<f64>().unwrap_or(150.0);

    let low_pnl_threshold_str = env::var("AI_OPTIMIZER_LOW_PNL_THRESHOLD").unwrap_or_else(|_| "-20.0".to_string());
    let low_pnl_threshold = low_pnl_threshold_str.parse::<f64>().unwrap_or(-20.0);

    if avg_pnl_percent > high_pnl_threshold {
        new_sell_multiplier *= 1.1;
        new_buy_amount_sol *= 1.2; // Augmenter le montant d'achat si la stratégie est très rentable
    } else if avg_pnl_percent < low_pnl_threshold {
        new_trailing_stop *= 0.9;
        new_buy_amount_sol *= 0.8; // Réduire le montant d'achat si la stratégie est perdante
    }

    OptimizedParams {
        buy_amount_sol: new_buy_amount_sol.clamp(0.001, 0.1), // Limiter entre 0.001 et 0.1 SOL
        sell_multiplier: new_sell_multiplier.clamp(1.2, 5.0),
        trailing_stop_percent: new_trailing_stop.clamp(0.05, 0.25),
    }
}