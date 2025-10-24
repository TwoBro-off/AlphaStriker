use crate::core::BotCore;
use crate::db::{Database, StrategySnapshot, TradeRecord};
use crate::{decision_engine, parsing::extract_pool_keys_from_tx_internal, tx_builder};
use futures_util::{StreamExt, SinkExt};
use solana_client::{nonblocking::rpc_client::RpcClient, rpc_config::RpcTransactionConfig};
use solana_sdk::commitment_config::CommitmentConfig;
use solana_sdk::pubkey::Pubkey;
use solana_sdk::program_pack::Pack;
use spl_token::state::Mint;
use solana_transaction_status::UiTransactionEncoding;
use serde_json::json;
use std::sync::atomic::{AtomicI64, Ordering};
use std::str::FromStr;
use std::sync::Arc;
use std::time::Duration;
use tokio::time::sleep;
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};
use tracing::{error, info, warn};
use uuid::Uuid;

const RAYDIUM_LIQUIDITY_POOL_V4: &str = "675kPX9MHTjS2zt1qfr1NYHuzeLXfQM9H24wFSUt1Mp8";
const SOL_MINT: &str = "So11111111111111111111111111111111111111112";

pub async fn run_log_scanner(bot_core: BotCore, rpc_ws_url: String, heartbeat: Arc<AtomicI64>) {
    info!("[Log Scanner] Démarrage du scanner de logs sur {}", rpc_ws_url);
    loop {
        if !bot_core.is_running() {
            info!("[Log Scanner] Arrêt détecté. Le scanner de logs se termine.");
            break;
        }

        match connect_async(&rpc_ws_url).await {
            Ok((ws_stream, _)) => {
                info!("[Log Scanner] Connecté au WebSocket RPC.");
                let (mut write, mut read) = ws_stream.split();

                let subscribe_request = json!({
                    "jsonrpc": "2.0",
                    "id": 1,
                    "method": "logsSubscribe",
                    "params": [
                        { "mentions": [RAYDIUM_LIQUIDITY_POOL_V4] },
                        { "commitment": "processed" }
                    ]
                });

                if let Err(e) = write.send(Message::Text(subscribe_request.to_string())).await {
                    error!("[Log Scanner] Échec de l'abonnement aux logs: {}", e);
                    sleep(Duration::from_secs(10)).await;
                    continue;
                }

                while let Some(Ok(msg)) = read.next().await {
                    heartbeat.store(chrono::Utc::now().timestamp(), Ordering::Relaxed);
                    if !bot_core.is_running() {
                        info!("[Log Scanner] Arrêt détecté. Déconnexion du WebSocket.");
                        break;
                    }

                    if let Message::Text(text) = msg {
                        if let Ok(value) = serde_json::from_str::<serde_json::Value>(&text) {
                            if let Some(logs) = value.pointer("/params/result/value/logs") {
                                let is_initialize_tx = logs.as_array().map_or(false, |log_lines| {
                                    log_lines.iter().any(|line| line.as_str().map_or(false, |s| s.contains("initialize2")))
                                });

                                if is_initialize_tx {
                                    if let Some(signature) = value.pointer("/params/result/value/signature").and_then(|s| s.as_str()) {
                                        info!("[Log Scanner] Nouvelle pool potentielle détectée, signature: {}", signature);
                                        info!("[Log Scanner] Logs complets: {:?}", logs);
                                        let core_clone = bot_core.clone();
                                        let signature_clone = signature.to_string();
                                        tokio::spawn(async move {
                                            process_signature(core_clone, signature_clone).await;
                                        });
                                    } else {
                                        warn!("[Log Scanner] Transaction initialize2 détectée mais signature manquante");
                                    }
                                }
                            }
                        }
                    }
                }
                warn!("[Log Scanner] Le flux WebSocket s'est terminé. Reconnexion...");
            }
            Err(e) => {
                error!("[Log Scanner] Échec de la connexion au WebSocket: {}. Nouvelle tentative dans 10s.", e);
                sleep(Duration::from_secs(10)).await;
            }
        }
    }
}

async fn process_signature(bot_core: BotCore, signature: String) {
    let rpc_client = RpcClient::new(bot_core.rpc_url());
    let config = RpcTransactionConfig {
        encoding: Some(UiTransactionEncoding::Json),
        commitment: Some(CommitmentConfig::processed()),
        max_supported_transaction_version: Some(0),
    };

    let signature_parsed = match signature.parse() {
        Ok(s) => s,
        Err(e) => {
            error!("[Log Scanner] Signature invalide {}: {}", signature, e);
            return;
        }
    };

    match rpc_client.get_transaction_with_config(&signature_parsed, config).await {
        Ok(tx_with_meta) => match serde_json::to_string(&tx_with_meta) {
            Ok(json_str) => {
                if let Some(pool_keys) = extract_pool_keys_from_tx_internal(&json_str) {
                    if let Some(token_mint) = pool_keys.get("base_mint").filter(|&m| m != SOL_MINT).or_else(|| pool_keys.get("quote_mint").filter(|&m| m != SOL_MINT)) {
                        info!("[Log Scanner] Pool Raydium extraite pour le token: {}", token_mint);
                        handle_new_token(bot_core, token_mint.clone()).await;
                    }
                }
            }
            Err(e) => error!("[Log Scanner] Erreur de sérialisation de la transaction {}: {}", signature, e),
        },
        Err(e) => {
            warn!("[Log Scanner] Impossible de récupérer la transaction {}: {}", signature, e);
            if e.to_string().contains("429") {
                warn!("[Log Scanner] Rate limit détecté. Attente de 2 secondes...");
                sleep(Duration::from_secs(2)).await;
            } else if e.to_string().contains("null") {
                warn!("[Log Scanner] Transaction non trouvée ou expirée: {}", signature);
            } else {
                warn!("[Log Scanner] Erreur RPC: {}. Attente de 1 seconde...", e);
                sleep(Duration::from_secs(1)).await;
            }
        }
    }
}

pub async fn handle_new_token(bot_core: BotCore, token_mint: String) {
    let current_positions = bot_core.trading_core().get_all_positions_internal().len();
    let max_positions = bot_core.get_max_positions().await;
    if current_positions >= max_positions {
        info!("[Handler] Limite de positions ({}/{}) atteinte. Achat pour {} ignoré.", current_positions, max_positions, token_mint);
        return;
    }

    let rpc_client = RpcClient::new(bot_core.rpc_url());
    let token_mint_pubkey = match Pubkey::from_str(&token_mint) {
        Ok(p) => p,
        Err(e) => {
            error!("[Handler] Token mint address invalide {}: {}", token_mint, e);
            return;
        }
    };

    let actual_token_decimals = match rpc_client.get_account(&token_mint_pubkey).await {
        Ok(account) => {
            match Mint::unpack_from_slice(&account.data) {
                Ok(mint_account) => mint_account.decimals as f64,
                Err(e) => { warn!("[Handler] Impossible de décompresser le compte Mint pour {}: {}. Utilisation de 6 décimales par défaut.", token_mint, e); 6.0 }
            }
        },
        Err(e) => { warn!("[Handler] Impossible de récupérer le compte Mint pour {}: {}. Utilisation de 6 décimales par défaut.", token_mint, e); 6.0 }
    };

    let buy_amount_sol = bot_core.trading_core().buy_amount_sol;
    let run_mode = bot_core.get_run_mode().await;

    if run_mode == crate::core::RunMode::Real {
        // --- LOGIQUE RÉELLE ---
        let trusted_creators = bot_core.trusted_creators();
        if !matches!(decision_engine::perform_security_checks(&bot_core, &token_mint, trusted_creators).await, decision_engine::SecurityCheckResult::Safe) {
            warn!("[SÉCURITÉ] Le token {} a échoué les vérifications de sécurité. Achat annulé.", token_mint);
            return;
        }

        let buy_amount_lamports = (buy_amount_sol * 1_000_000_000.0) as u64;
        let quote_url = format!("https://quote-api.jup.ag/v6/quote?inputMint={}&outputMint={}&amount={}&slippageBps=500", SOL_MINT, token_mint, buy_amount_lamports);

        match bot_core.http_client().get(&quote_url).send().await {
            Ok(quote_resp) if quote_resp.status().is_success() => {
                if let Ok(quote_json_str) = quote_resp.text().await {
                    info!("[Handler] Quote obtenu pour {}. Lancement du processus d'achat...", token_mint);
                    match bot_core.process_new_token_internal(&token_mint, &quote_json_str).await {
                        Ok(Some(signed_tx)) => {
                            info!("[Handler] Transaction signée obtenue. Envoi au réseau...");
                            match tx_builder::send_raw_transaction(bot_core.rpc_url().as_str(), &signed_tx).await {
                                Ok(_) => {
                                    info!("[Handler] Transaction envoyée avec succès pour {}! Enregistrement de la position...", token_mint);
                                    // Enregistrer la position après l'envoi, comme en mode démo
                                    if let Ok(quote_val) = serde_json::from_str::<serde_json::Value>(&quote_json_str) {
                                        if let Some(out_amount_str) = quote_val.get("outAmount").and_then(|v| v.as_str()) {
                                            if let Ok(out_amount_units) = out_amount_str.parse::<f64>() {
                                                let amount_tokens = out_amount_units / 10f64.powf(actual_token_decimals);
                                                if amount_tokens > 0.0 {
                                                    let buy_price = buy_amount_sol / amount_tokens;
                                                    let buy_timestamp = chrono::Utc::now().timestamp();
                                                    let trading_core = bot_core.trading_core();
                                                    let snapshot = StrategySnapshot {
                                                        buy_amount_sol: trading_core.buy_amount_sol,
                                                        sell_multiplier: trading_core.sell_multiplier,
                                                        trailing_stop_percent: trading_core.trailing_stop_percent,
                                                    };
                                                    let trade_id = Uuid::new_v4().to_string();
                                                    trading_core.add_position(trade_id.clone(), token_mint.clone(), buy_price, buy_amount_sol, amount_tokens, buy_timestamp, snapshot.clone());                                                
                                                    let initial_trade_record = TradeRecord {
                                                        id: trade_id,
                                                        token_address: token_mint.clone(),
                                                        buy_timestamp,
                                                        buy_price,
                                                        amount_sol: buy_amount_sol,
                                                        amount_tokens,
                                                        sell_timestamp: None, sell_price: None, pnl_sol: None, pnl_percent: None,
                                                        strategy_snapshot: Some(snapshot),
                                                        run_mode: "REAL".to_string(),
                                                    };
                                                    let db_clone = bot_core.db(); //NOSONAR
                                                    tokio::spawn(async move { db_clone.save_trade_with_retry(initial_trade_record).await; });
                                                }
                                            }
                                        }
                                    }
                                }
                                Err(e) => error!("[Handler] Échec de l'envoi de la transaction pour {}: {}", token_mint, e),
                            }
                        }
                        Ok(None) => info!("[Handler] Achat non exécuté pour {} (déjà détenu).", token_mint),
                        Err(e) => error!("[Handler] Erreur lors du traitement du token {}: {}", token_mint, e),
                    }
                }
            }
            _ => error!("[Handler] Impossible d'obtenir un devis de Jupiter pour {}", token_mint),
        }
    } else {
        // --- LOGIQUE DE SIMULATION ---
        info!("[SIMULATION] Tentative d'achat simulé pour le token: {}", token_mint);
        let buy_amount_lamports = (buy_amount_sol * 1_000_000_000.0) as u64;
        let quote_url = format!("https://quote-api.jup.ag/v6/quote?inputMint={}&outputMint={}&amount={}&slippageBps=500", SOL_MINT, &token_mint, buy_amount_lamports);

        match bot_core.http_client().get(&quote_url).send().await {
            Ok(quote_resp) if quote_resp.status().is_success() => {
                if let Ok(quote_json_str) = quote_resp.text().await {
                    if let Ok(quote_val) = serde_json::from_str::<serde_json::Value>(&quote_json_str) {
                        if let Some(out_amount_str) = quote_val.get("outAmount").and_then(|v| v.as_str()) {
                            if let Ok(out_amount_units) = out_amount_str.parse::<f64>() {
                                let amount_tokens = out_amount_units / 10f64.powf(actual_token_decimals);

                                if amount_tokens > 0.0 {
                                    let buy_price = buy_amount_sol / amount_tokens;
                                    let buy_timestamp = chrono::Utc::now().timestamp();
                                    let trading_core = bot_core.trading_core();
                                    let snapshot = StrategySnapshot {
                                        buy_amount_sol: trading_core.buy_amount_sol,
                                        sell_multiplier: trading_core.sell_multiplier,
                                        trailing_stop_percent: trading_core.trailing_stop_percent,
                                    };
                                    let trade_id = Uuid::new_v4().to_string();

                                    trading_core.add_position(trade_id.clone(), token_mint.clone(), buy_price, buy_amount_sol, amount_tokens, buy_timestamp, snapshot.clone());

                                    let initial_trade_record = TradeRecord {
                                        id: trade_id,
                                        token_address: token_mint.clone(),
                                        buy_timestamp,
                                        buy_price,
                                        amount_sol: buy_amount_sol,
                                        amount_tokens,
                                        sell_timestamp: None, sell_price: None, pnl_sol: None, pnl_percent: None,
                                        strategy_snapshot: Some(snapshot),
                                        run_mode: "DEMO".to_string(),
                                    };
                                    let db_clone = bot_core.db();
                                tokio::spawn(async move { db_clone.save_trade_with_retry(initial_trade_record).await; });
                                    info!("[SIMULATION] Position ouverte pour {} @ {:.6} SOL.", token_mint, buy_price);
                                }
                            }
                        }
                    }
                }
            }
            _ => error!("[SIMULATION] Impossible d'obtenir un devis de Jupiter pour {}", token_mint),
        }
    }
}
