use axum::{
    extract::{Query, State},
    http::{Method, StatusCode}, //NOSONAR
    response::{IntoResponse, Json},
    routing::{get, post},
    Router,
};
use crate::{core::{BotCore, RunMode}, state_manager::StrategyParams, test_tokens};
use serde::Deserialize;
use solana_sdk::pubkey::Pubkey;
use std::{net::SocketAddr, sync::{Arc, atomic::{AtomicI64, Ordering}}}; //NOSONAR
use tracing::{error, info, warn, instrument};
use tower_http::cors::{Any, CorsLayer};
use chrono;
use bs58;
use dotenvy;
use solana_client;
use serde_json;
use tokio;

pub mod creator_tracker;
pub mod db;
pub mod logger;
pub mod ia_optimizer;
pub mod metrics;
pub mod bot_loop;
pub mod core;
pub mod decision_engine;
pub mod log_scanner;
pub mod parsing;
pub mod state_manager;
pub mod tx_builder;
pub mod test_tokens;

#[derive(Clone)]
struct AppState {
    bot_core: Arc<BotCore>,
    heartbeat: Arc<AtomicI64>,
}


#[derive(Deserialize)]
struct StartParams {
    mode: String,
}


#[tokio::main]
async fn main() {
    let _log_guard = logger::init_tracing("logs").expect("Failed to initialize tracing");

    dotenvy::dotenv().expect("Impossible de charger le fichier .env");
    info!("Variables d'environnement chargées.");

    let bot_core = match init_bot_core() {
        Ok(core) => Arc::new(core),
        Err(e) => {
            error!("Échec de l'initialisation du BotCore: {}", e);
            return;
        }
    };
    let heartbeat = Arc::new(AtomicI64::new(chrono::Utc::now().timestamp()));

    // Démarrer le tracker de créateurs uniquement si le mode initial est DEMO.
    if bot_core.initial_run_mode == RunMode::Demo {
        let creator_tracker_core_clone = bot_core.clone();
        tokio::spawn(creator_tracker::run_creator_tracker((*creator_tracker_core_clone).clone()));
        info!("[AI] Le tracker de créateurs est activé (mode DEMO).");
        // L'optimiseur IA est démarré via l'API `start_bot` et non au lancement global.
    }

    info!("Tâches de fond du bot démarrées.");

    let watchdog_heartbeat = heartbeat.clone();
    tokio::spawn(async move {
        loop {
            tokio::time::sleep(std::time::Duration::from_secs(60)).await;
            let now = chrono::Utc::now().timestamp();
            let last_hb = watchdog_heartbeat.load(Ordering::Relaxed); //NOSONAR
            if now - last_hb > 120 { // Si pas de signe de vie depuis 2 minutes
                warn!("[WATCHDOG] Aucune activité détectée depuis plus de 2 minutes. Le bot est peut-être bloqué.");
            }
        }
    });

    let metrics_clone = bot_core.get_metrics();
    tokio::spawn(async move {
        let metrics_app = Router::new()
            .route("/metrics", get(metrics::serve_metrics))
            .with_state(metrics_clone);
        let metrics_addr: SocketAddr = ([0, 0, 0, 0], 9090).into();
        info!("Serveur de métriques Prometheus démarré sur http://{}", metrics_addr);
        let listener_metrics = tokio::net::TcpListener::bind(metrics_addr).await.unwrap();
        axum::serve(listener_metrics, metrics_app.into_make_service()).await.unwrap();
    });

    let app_state = AppState {
        bot_core: bot_core.clone(),
        heartbeat: heartbeat.clone(),
    };

    let app = Router::new()
        .route("/", get(root))
        .route("/api/bot/status", get(get_bot_status))
        .route("/api/bot/start", post(start_bot))
        .route("/api/bot/stop", post(stop_bot))
        .route("/api/simulation/dashboard", get(get_simulation_dashboard))
        .route("/api/bot/activity", get(get_activity_log))
        .route("/api/bot/readiness", get(get_readiness))
        .route("/api/pre-flight-check", get(get_readiness)) // Alias for compatibility
        .route("/api/bot/optimizer/status", get(get_ai_status)) // Alias pour la cohérence frontend
        .route("/api/ai/status", get(get_ai_status))
        .route("/api/strategy/settings", get(get_strategy_settings))
        .route("/api/strategy/settings", post(update_strategy_settings))
        .route("/api/test", post(test_bot))
        .with_state(app_state)
        .layer(
            CorsLayer::new()
                .allow_methods([Method::GET, Method::POST])
                .allow_headers(vec![axum::http::header::CONTENT_TYPE])
                .allow_origin(Any),
        );

    let addr: SocketAddr = ([0, 0, 0, 0], 8000).into();
    info!("Serveur web démarré sur {}", addr);
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app.into_make_service()).await.unwrap();
}

fn init_bot_core() -> Result<BotCore, String> {
    let buy_amount_sol: f64 = std::env::var("BUY_AMOUNT_SOL").map_err(|e| e.to_string())?.parse().map_err(|e: std::num::ParseFloatError| e.to_string())?;
    let sell_multiplier: f64 = std::env::var("SELL_MULTIPLIER").map_err(|e| e.to_string())?.parse().map_err(|e: std::num::ParseFloatError| e.to_string())?;
    let trailing_stop_percent: f64 = std::env::var("TRAILING_STOP_PERCENT").map_err(|e| e.to_string())?.parse().map_err(|e: std::num::ParseFloatError| e.to_string())?;
    let private_key = std::env::var("PRIVATE_KEY").map_err(|_| "PRIVATE_KEY non définie dans le .env".to_string())?;
    let rpc_url = std::env::var("RPC_URL").map_err(|_| "RPC_URL non définie dans le .env".to_string())?;
    let helius_api_key = std::env::var("HELIUS_API_KEY").map_err(|_| "HELIUS_API_KEY non définie dans le .env".to_string())?;

    let trusted_creators_str = std::env::var("TRUSTED_CREATORS").unwrap_or_default();
    let trusted_creators: Vec<Pubkey> = trusted_creators_str
        .split(',')
        .filter_map(|s| s.trim().parse::<Pubkey>().ok())
        .collect();

    info!("{} créateurs de confiance chargés.", trusted_creators.len());

    let max_positions_real: usize = std::env::var("MAX_POSITIONS_REAL").unwrap_or_else(|_| "10".to_string()).parse().unwrap_or(10);
    let max_positions_demo: usize = std::env::var("MAX_POSITIONS_DEMO").unwrap_or_else(|_| "128".to_string()).parse().unwrap_or(128);
    
    // Le mode DEMO est maintenant le mode par défaut si RUN_MODE n'est pas spécifié.
    let run_mode_str = std::env::var("RUN_MODE").unwrap_or_else(|_| "DEMO".to_string()).to_uppercase();
    let run_mode = match run_mode_str.as_str() {
        "REAL" => RunMode::Real,
        _ => RunMode::Demo,
    };
    info!("Mode de fonctionnement: {:?}", run_mode);

    let final_private_key = if run_mode == RunMode::Real {
        // En mode REAL, la clé privée est obligatoire et doit être valide.
        if private_key.is_empty() {
            return Err("PRIVATE_KEY est obligatoire pour le mode REAL.".to_string());
        }
        let key_bytes = bs58::decode(&private_key)
            .into_vec()
            .map_err(|e| format!("PRIVATE_KEY invalide (caractère invalide): {}", e))?;
        solana_sdk::signature::Keypair::try_from(key_bytes.as_slice())
            .map_err(|e| format!("PRIVATE_KEY invalide (longueur incorrecte): {}", e))?;
        private_key
    } else {
        // En mode DEMO, si la clé est absente ou invalide, on en génère une nouvelle.
        bs58::decode(&private_key).into_vec()
            .ok()
            .and_then(|bytes| solana_sdk::signature::Keypair::try_from(bytes.as_slice()).ok())
            .map(|_| private_key) // La clé fournie est valide, on l'utilise.
            .unwrap_or_else(|| {
                warn!("Clé privée non fournie ou invalide en mode DEMO. Génération d'une clé temporaire.");
                solana_sdk::signature::Keypair::new().to_base58_string()
            })
    };

    if rpc_url.is_empty() {
        return Err("RPC_URL ne peut pas être vide".to_string());
    }

    Ok(BotCore::new(
        buy_amount_sol,
        sell_multiplier,
        trailing_stop_percent,
        final_private_key,
        rpc_url,
        helius_api_key,
        trusted_creators,
        "alphastriker_db",
        run_mode,
        max_positions_real,
        max_positions_demo,
    ))
}

async fn root() -> &'static str {
    "Welcome to AlphaStriker API (100% Rust)"
}

async fn get_strategy_settings(State(state): State<AppState>) -> impl IntoResponse {
    let params = state.bot_core.trading_core().get_strategy_params();
    (
        StatusCode::OK,
        Json(params),
    )
}

async fn get_bot_status(State(state): State<AppState>) -> Json<serde_json::Value> {
    let is_running = state.bot_core.is_running();
    let is_responsive = chrono::Utc::now().timestamp() - state.heartbeat.load(Ordering::Relaxed) < 120;
    let current_mode = format!("{:?}", state.bot_core.get_run_mode().await).to_lowercase();
    Json(serde_json::json!({
        "is_running": is_running,
        "current_mode": current_mode,
        "is_offline": !is_responsive,
    }))
}

async fn get_simulation_dashboard(State(state): State<AppState>) -> Json<serde_json::Value> {
    // Cette fonction retourne maintenant les données de la DB, peu importe le mode.
    let positions = state.bot_core.trading_core().get_all_positions_internal(); // Positions actuellement ouvertes
    
    let (trade_history_for_chart, total_trades_completed, profit_loss_cumulated) = match state.bot_core.db().get_all_trades() {
        Ok(trades) => {
            let mut cumulative_pnl = 0.0;
            let mut sorted_trades = trades;
            // Trier par date de vente (ou d'achat si non vendu) pour le graphique
            sorted_trades.sort_by_key(|t| t.sell_timestamp.unwrap_or(t.buy_timestamp));

            let completed_trades: Vec<_> = sorted_trades.iter().filter(|t| t.pnl_sol.is_some()).collect();
            let total_trades = completed_trades.len();
            let total_pnl = completed_trades.iter().filter_map(|t| t.pnl_sol).sum();

            let history_data = completed_trades.into_iter().map(|trade| {
                cumulative_pnl += trade.pnl_sol.unwrap_or(0.0);
                serde_json::json!({
                    "timestamp": trade.sell_timestamp.unwrap_or(trade.buy_timestamp),
                    "cumulative_pnl": cumulative_pnl,
                })
            }).collect::<Vec<_>>();

            (history_data, total_trades, total_pnl)
        },
        Err(_) => (vec![], 0, 0.0),
    };

    Json(serde_json::json!({
        "profit_loss_sol": profit_loss_cumulated,
        "total_trades": total_trades_completed,
        "held_tokens_count": positions.len(),
        "trade_history": trade_history_for_chart
    }))
}

async fn start_bot(
    State(app_state): State<AppState>,
    Query(params): Query<StartParams>,
) -> impl IntoResponse {
    if app_state.bot_core.is_running() {
        return (StatusCode::BAD_REQUEST, Json(serde_json::json!({"detail": "Le bot est déjà en cours d'exécution."}))).into_response();
    }
    info!("Requête de démarrage reçue pour le mode: {}", params.mode);

    let run_mode = match params.mode.to_uppercase().as_str() {
        "REAL" => RunMode::Real,
        "SIMULATION" => RunMode::Demo,
        _ => return (StatusCode::BAD_REQUEST, Json(serde_json::json!({"detail": "Mode invalide. Utilisez 'real' ou 'simulation'."}))).into_response(),
    };
    app_state.bot_core.set_run_mode(run_mode).await;
    app_state.bot_core.set_running_status(true);

    let trading_core_clone = app_state.bot_core.clone();
    let heartbeat_clone = app_state.heartbeat.clone();
    tokio::spawn(async move {
        let rpc_ws_url = trading_core_clone.rpc_url().replace("http", "ws");
        tokio::select! { //NOSONAR
            _ = crate::log_scanner::run_log_scanner((*trading_core_clone).clone(), rpc_ws_url, heartbeat_clone.clone()) => { info!("Le scanner de logs s'est terminé."); },
            _ = crate::bot_loop::run_bot_loop((*trading_core_clone).clone(), heartbeat_clone) => {},
        }
    });

    // Démarrer l'optimiseur IA uniquement en mode simulation
    if run_mode == RunMode::Demo {
        let ai_core_clone = app_state.bot_core.clone();
        tokio::spawn(ia_optimizer::run_ai_optimizer((*ai_core_clone).clone()));
        info!("[AI] L'optimiseur IA est activé (mode DEMO).");
    }

    (StatusCode::OK, Json(serde_json::json!({"status": "démarrage"}))).into_response()
}

#[derive(Deserialize)]
struct TestParams {
    token_mint: Option<String>,
}

async fn test_bot(
    State(app_state): State<AppState>,
    Query(params): Query<TestParams>,
) -> impl IntoResponse {
    info!("Requête de test reçue avec les paramètres : {:?}", params);

    let test_core_clone = app_state.bot_core.clone();
    if let Some(token_mint) = params.token_mint {
        info!("[TEST] Déclenchement d'un achat simulé pour le token : {}", token_mint);
        tokio::spawn(async move {
            crate::log_scanner::handle_new_token(test_core_clone, token_mint).await;
        });
    } else {
        tokio::spawn(async move { test_tokens::run_test_mode((*test_core_clone).clone()).await; });
    }

    (StatusCode::OK, Json(serde_json::json!({"status": "test_démarré"}))).into_response()
}

async fn stop_bot(State(state): State<AppState>) -> impl IntoResponse {
    if !state.bot_core.is_running() {
        return (StatusCode::BAD_REQUEST, Json(serde_json::json!({"detail": "Le bot n'est pas en cours d'exécution."}))).into_response();
    }
    info!("Requête d'arrêt reçue.");
    state.bot_core.set_running_status(false); // Ceci arrêtera les boucles de fond
    (StatusCode::OK, Json(serde_json::json!({"status": "arrêt en cours"}))).into_response()
}

async fn get_activity_log(State(state): State<AppState>) -> Json<serde_json::Value> {
    let mut trades = match state.bot_core.db().get_recent_trades(50) {
        Ok(trades) => trades,
        Err(e) => {
            error!("Impossible de lire l'historique des trades pour le journal d'activite: {}", e);
            return Json(serde_json::json!([]));
        }
    };
    // L'API retourne maintenant directement la liste des trades. Le tri est déjà fait par la DB.
    trades.sort_by_key(|t| std::cmp::Reverse(t.buy_timestamp));
    Json(serde_json::to_value(trades).unwrap_or_default())
}

async fn get_ai_status(State(state): State<AppState>) -> Json<serde_json::Value> {
    let is_ai_active = state.bot_core.is_running() && matches!(state.bot_core.get_run_mode().await, RunMode::Demo);
    let params = state.bot_core.trading_core().get_strategy_params();
    let current_mode = format!("{:?}", state.bot_core.get_run_mode().await).to_uppercase();

    Json(serde_json::json!({
        "is_active": is_ai_active,
        "mode": current_mode,
        "optimizations_count": 0, // Placeholder
        "next_cycle": "N/A", // Placeholder
        "current_sell_multiplier": params.sell_multiplier,
        "current_trailing_stop": params.trailing_stop_percent,
        "current_trade_amount": params.buy_amount_sol,
        "last_optimization": null
    }))
}

async fn get_readiness(State(state): State<AppState>) -> Json<serde_json::Value> {
    // Vérifications de base
    let private_key_set = !state.bot_core.get_wallet_pk().is_empty();
    let helius_key_set = !state.bot_core.helius_api_key().is_empty();

    // Vérifications réseau
    let rpc_client = solana_client::nonblocking::rpc_client::RpcClient::new(state.bot_core.rpc_url().clone());

    // Exécuter les vérifications réseau en parallèle
    let (rpc_health_result, balance_check_result) = tokio::join!(
        rpc_client.get_health(),
        async {
            if private_key_set {
                if let Ok(pubkey) = state.bot_core.get_wallet_pubkey() {
                    match rpc_client.get_balance(&pubkey).await {
                        Ok(balance) => Ok((true, balance)), // pk_valid, balance
                        Err(e) => {
                            error!("Échec de la récupération du solde pour la clé publique {}: {}", pubkey, e);
                            Err(true) // pk_valid, mais erreur de solde
                        }
                    }
                } else {
                    Ok((false, 0)) // pk_invalid
                }
            } else {
                Ok((false, 0)) // pk non définie
            }
        }
    );

    let rpc_ok = rpc_health_result.is_ok();
    let (pk_valid, balance_ok, balance_sol) = match balance_check_result {
        Ok((pk_v, balance)) => {
            let bal_sol = (balance as f64) / 1_000_000_000.0;
            (pk_v, balance > 0, bal_sol)
        },
        Err(pk_v) => {
            // Erreur de récupération du solde, mais la clé peut être valide
            (pk_v, false, 0.0)
        }
    };

    let is_ready = pk_valid && rpc_ok && balance_ok;
    Json(serde_json::json!({
        "is_ready": is_ready,
        "checks": {
            "private_key_set": private_key_set,
            "rpc_connection_ok": rpc_ok,
            "private_key_valid": pk_valid,
            "helius_key_set": helius_key_set,
            "initial_balance_ok": balance_ok,
            "balance_sol": balance_sol,
        }
    }))
}

#[instrument(skip(state, new_params))]
async fn update_strategy_settings(
    State(state): State<AppState>,
    Json(new_params): Json<StrategyParams>,
) -> impl IntoResponse {
    info!("Mise à jour des paramètres de stratégie: {:?}", new_params);

    // Validation
    if new_params.buy_amount_sol <= 0.0 || new_params.sell_multiplier <= 1.0 || new_params.trailing_stop_percent <= 0.0 || new_params.trailing_stop_percent >= 1.0 {
        return (StatusCode::BAD_REQUEST, Json(serde_json::json!({"detail": "Paramètres de stratégie invalides."}))).into_response();
    }

    let mut trading_core = state.bot_core.trading_core();
    trading_core.set_strategy_params(
        new_params.buy_amount_sol,
        new_params.sell_multiplier,
        new_params.trailing_stop_percent,
    );

    info!("Paramètres de stratégie mis à jour avec succès.");
    (StatusCode::OK, Json(serde_json::json!({"status": "ok"}))).into_response()
}