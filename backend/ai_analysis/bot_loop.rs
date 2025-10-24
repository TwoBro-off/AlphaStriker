use crate::core::BotCore;
use crate::tx_builder;
use crate::db::{Database, TradeRecord};
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_sdk::program_pack::Pack;
use spl_token::state::Mint;
use tracing::{info, warn, error}; // Use tracing for logging
use serde::Deserialize;
use std::{str::FromStr, time::Duration, sync::{Arc, atomic::{AtomicI64, Ordering}}};
use tokio::{time::sleep, env};
use std::collections::HashMap; // For PriceResponse data

#[derive(Debug, Clone, Copy)]
pub enum SaleReason {
    TakeProfit, StopLoss, Timeout,
}

const SOL_MINT: &str = "So11111111111111111111111111111111111111112";

#[derive(Deserialize, Debug, Default)]
struct PriceResponse {
    data: HashMap<String, PriceData>, // Changed to HashMap
}

#[derive(Deserialize, Debug)]
struct PriceData {
    price: f64,
}

pub async fn run_bot_loop(bot_core: BotCore, heartbeat: Arc<AtomicI64>) {
    info!("[Bot Loop Spawner] Démarrage du gestionnaire de surveillance.");
    let mut monitored_tokens = std::collections::HashSet::new();

    loop {
        heartbeat.store(chrono::Utc::now().timestamp(), Ordering::Relaxed);
        if !bot_core.is_running() {
            info!("[Bot Loop Spawner] Arrêt détecté. Le spawner de surveillance se termine.");
            break;
        }

        sleep(Duration::from_secs(5)).await;
 
        let positions = bot_core.trading_core().get_all_positions_internal();
        for (token_address, _position) in positions {
            if !monitored_tokens.contains(&token_address) {
                info!("[Bot Loop Spawner] Nouvelle position détectée: {}. Lancement de la surveillance.", token_address);
                monitored_tokens.insert(token_address.clone());
                let core_clone = bot_core.clone();
                tokio::spawn(monitor_position(core_clone, token_address));
            }
        }
        let position_keys: std::collections::HashSet<String> = bot_core.trading_core().get_all_positions_internal().into_iter().map(|(k, _)| k).collect();
        monitored_tokens.retain(|token| position_keys.contains(token));
    }
}
async fn monitor_position(bot_core: BotCore, token_address: String) {
    info!("[Monitor] Démarrage de la surveillance pour {}", token_address);
    let scan_interval_secs_str = std::env::var("POSITION_SCAN_INTERVAL_SECS").unwrap_or_else(|_| "2".to_string());
    let scan_interval_secs = scan_interval_secs_str.parse::<u64>().unwrap_or(2);
    let scan_interval = Duration::from_secs(scan_interval_secs);

    while let Some(position) = bot_core.trading_core().get_position(&token_address) {
        if !bot_core.is_running() {
            info!("[Monitor] Arrêt détecté pour {}. La surveillance se termine.", token_address);
            break;
        }

        sleep(scan_interval).await;

        let current_timestamp = chrono::Utc::now().timestamp();
        let time_elapsed_secs = current_timestamp - position.buy_timestamp;
        let max_duration_secs = bot_core.trading_core().max_trade_duration_secs;

        if time_elapsed_secs > max_duration_secs as i64 {
            info!("[Monitor] Le token {} est détenu depuis plus de {}s. Vente forcée (timeout).", token_address, max_duration_secs);
            if let Err(e) = execute_sale(bot_core.clone(), position, SaleReason::Timeout).await {
                error!("[Monitor] Échec de l'exécution de la vente forcée pour {}: {}", token_address, e);
            }
            break;
        }
 
        let mut price_response_result: Option<Result<PriceResponse, reqwest::Error>> = None;
        let mut attempts = 0;
        let max_attempts = 3;
        let backoff_delays = [Duration::from_secs(1), Duration::from_secs(3), Duration::from_secs(5)];

        while attempts < max_attempts {
            let url = format!("https://price.jup.ag/v4/price?ids={}", token_address);
            match bot_core.http_client().get(&url).send().await {
                Ok(resp) => {
                    if resp.status().is_success() {
                        price_response_result = Some(resp.json().await);
                        break; // Success, exit retry loop
                    } else {
                        warn!("[Monitor] API Jupiter a retourné un statut non-succès {} pour {}. Tentative {}/{}...", resp.status(), token_address, attempts + 1, max_attempts);
                    }
                },
                Err(e) => {
                    warn!("[Monitor] Erreur réseau lors de la récupération du prix pour {}: {}. Tentative {}/{}...", token_address, e, attempts + 1, max_attempts);
                }
            }

            attempts += 1;
            if attempts < max_attempts {
                sleep(backoff_delays[attempts - 1]).await;
            }
        }

        if let Some(Ok(price_data)) = price_response_result {
            if let Some(data) = price_data.data.get(&token_address) {
                let current_price = data.price;
                let decision = bot_core.trading_core().evaluate_for_sale(&token_address, current_price);
                let sale_reason = match decision {
                    crate::state_manager::SaleDecision::SellTakeProfit => Some(SaleReason::TakeProfit),
                    crate::state_manager::SaleDecision::SellTrailingStop => Some(SaleReason::StopLoss),
                    _ => None,
                };

                if let Some(reason) = sale_reason {
                    info!("[Monitor] Décision de vente pour {} à {}. Raison: {:?}", token_address, current_price, decision);
                    if let Err(e) = execute_sale(bot_core.clone(), position, reason).await {
                        error!("[Monitor] Échec de l'exécution de la vente pour {}: {}", token_address, e);
                    }
                    break;
                }
            }
        } else { // This covers both None and Err cases from price_response_result
            warn!("[Monitor] Impossible de récupérer le prix pour {}", token_address);
        }
    }
    info!("[Monitor] Arrêt de la surveillance pour {} (position vendue ou supprimée).", token_address);
}

async fn execute_sale(bot_core: BotCore, position: crate::state_manager::Position, _reason: SaleReason) -> Result<(), String> {
    let token_address = position.token_address.clone();
    info!("[Bot Loop] Début du processus de vente pour {}", token_address);

    let rpc_client = RpcClient::new(bot_core.rpc_url());
    let token_mint_pubkey = match solana_sdk::pubkey::Pubkey::from_str(&token_address) {
        Ok(p) => p,
        Err(e) => {
            return Err(format!("[Sale] Adresse de token invalide {}: {}", token_address, e));
        }
    };

    let actual_token_decimals = match rpc_client.get_account(&token_mint_pubkey).await {
        Ok(account) => {
            match Mint::unpack_from_slice(&account.data) {
                Ok(mint_account) => mint_account.decimals as f64,
                Err(e) => { warn!("[Sale] Impossible de décompresser le compte Mint pour {}: {}. Utilisation de 6 décimales par défaut.", token_address, e); 6.0 }
            }
        },
        Err(e) => { warn!("[Sale] Impossible de récupérer le compte Mint pour {}: {}. Utilisation de 6 décimales par défaut.", token_address, e); 6.0 }
    };

    let amount_units = (position.amount_tokens * 10f64.powf(actual_token_decimals)) as u64;

    let quote_url = format!(
        "https://quote-api.jup.ag/v6/quote?inputMint={}&outputMint={}&amount={}&slippageBps=500",
        &token_address, SOL_MINT, amount_units
    );
    let quote_resp = bot_core.http_client().get(&quote_url).send().await.map_err(|e| e.to_string())?;
    if !quote_resp.status().is_success() {
        return Err(format!("Échec de l'obtention du devis de vente pour {}", token_address));
    }
    let quote_json = quote_resp.text().await.map_err(|e| e.to_string())?;

    let signed_tx = tx_builder::build_and_sign_jupiter_swap_internal(&bot_core.http_client(), quote_json.clone(), bot_core.get_wallet_pk(), bot_core.rpc_url(), None).await.map_err(|e| e.to_string())?; // Use getter
    tx_builder::send_raw_transaction(bot_core.rpc_url().as_str(), &signed_tx).await.map_err(|e| e.to_string())?;

    let sol_received = if let Ok(quote_val) = serde_json::from_str::<serde_json::Value>(&quote_json) {
        let out_amount_str = quote_val["outAmount"].as_str().unwrap_or("0");
        out_amount_str.parse::<f64>().unwrap_or(0.0) / 1_000_000_000.0
    } else {
        0.0
    };

    if position.amount_tokens <= 0.0 {
        warn!("[Bot Loop] Vente de {} annulée : quantité de tokens nulle ou négative.", token_address);
        return Ok(());
    }
    let sell_price = sol_received / position.amount_tokens;
    let pnl_sol = sol_received - position.amount_sol;
    let pnl_percent = (pnl_sol / position.amount_sol) * 100.0;

    let db_clone = bot_core.db();
    let bot_mode = bot_core.get_run_mode().await;
    let updated_record = TradeRecord {
        id: position.trade_id.clone(),
        token_address: position.token_address.clone(),
        buy_timestamp: position.buy_timestamp,
        buy_price: position.buy_price,
        amount_sol: position.amount_sol,
        amount_tokens: position.amount_tokens,
        sell_timestamp: Some(chrono::Utc::now().timestamp()),
        sell_price: Some(sell_price),
        pnl_sol: Some(pnl_sol),
        pnl_percent: Some(pnl_percent),
        strategy_snapshot: Some(position.strategy_snapshot.clone()),
        run_mode: format!("{:?}", bot_mode).to_uppercase(),
    };

    db_clone.save_trade_with_retry(updated_record).await;
    bot_core.trading_core().remove_position(&token_address);

    info!(
        "[TRADE COMPLETE] id={} token={} buy_price={} sell_price={} pnl_sol={:.6} pnl_percent={:.2}% mode={:?}",
        position.trade_id,
        token_address,
        position.buy_price,
        sell_price,
        pnl_sol,
        pnl_percent,
        bot_mode
    );
    
    Ok(())
}