use crate::core::{BotCore, RunMode};
use crate::db::{StrategySnapshot, TradeRecord};
use crate::metrics::Metrics;
use std::sync::Arc;
use tokio;
use uuid::Uuid;

#[tokio::test]
async fn test_trade_simulation_and_metrics() {
    // 1. Initialiser un BotCore de test avec une DB en mémoire ou temporaire
    let db_path = format!("test_db_{}", Uuid::new_v4());
    let bot_core = Arc::new(BotCore::new(
        0.01, 2.0, 0.15, 
        "your_private_key".to_string(), // Utiliser une clé de dev
        "https://api.devnet.solana.com".to_string(), // URL RPC pour le test
        "dummy_helius_key".to_string(),
        vec![],
        &db_path,
        RunMode::Demo,
        10,
        128,
    ));

    let mut tasks = vec![];
    let num_trades = 10;

    // 2. Simuler des achats concurrents
    for i in 0..num_trades {
        let core_clone = bot_core.clone();
        let task = tokio::spawn(async move {
            let trade_id = Uuid::new_v4().to_string();
            let token_address = format!("token_{}", i);
            let detection_ts = chrono::Utc::now().timestamp_millis();
            
            // Simuler une petite latence de détection
            tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
            
            let buy_ts = chrono::Utc::now().timestamp_millis();
            let buy_price = 100.0 + i as f64;
            let amount_sol = 0.01;
            let amount_tokens = 0.0001;

            let snapshot = StrategySnapshot {
                buy_amount_sol: amount_sol,
                sell_multiplier: 2.0,
                trailing_stop_percent: 0.15,
            };

            // Mettre à jour les métriques d'achat
            core_clone.metrics.record_buy(buy_ts - detection_ts);

            // Ajouter la position
            core_clone.trading_core().add_position(
                trade_id.clone(),
                token_address.clone(),
                buy_price,
                amount_sol,
                amount_tokens,
                buy_ts,
                snapshot.clone(),
            );

            // Simuler une vente après un certain temps
            tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
            let decision_ts = chrono::Utc::now().timestamp_millis();
            tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
            let sell_ts = chrono::Utc::now().timestamp_millis();

            let sell_price = buy_price * 1.5; // Simuler un P&L de 50%
            let sol_received = sell_price * amount_tokens;
            let pnl_sol = sol_received - amount_sol;

            // Mettre à jour les métriques de vente
            core_clone.metrics.record_sell(
                sell_ts - decision_ts,
                sell_ts - buy_ts,
                pnl_sol,
            );

            // Supprimer la position
            core_clone.trading_core().remove_position(&token_address);
        });
        tasks.push(task);
    }

    futures::future::join_all(tasks).await;

    // 3. Vérifier les métriques
    assert_eq!(bot_core.trading_core().get_all_positions_internal().len(), 0, "Toutes les positions devraient être fermées");
    
    let metrics_json = bot_core.metrics.as_json().await;
    
    let total_trades = metrics_json["total_trades_completed"].as_u64().unwrap();
    assert_eq!(total_trades, num_trades as u64, "Le nombre total de trades doit correspondre");

    let pnl = metrics_json["pnl_cumulated"].as_f64().unwrap();
    // PNL attendu: 10 trades * ( (100 * 1.5 * 0.0001) - 0.01 ) = 10 * (0.015 - 0.01) = 0.05
    // Le calcul est un peu plus complexe à cause de `i`
    assert!(pnl > 0.0, "Le PNL cumulé devrait être positif");

    let avg_buy_latency = metrics_json["buy_latency_avg_ms"].as_u64().unwrap();
    assert!(avg_buy_latency >= 50, "La latence d'achat moyenne devrait être d'au moins 50ms");
    // 4. Nettoyer la DB de test
    let _ = std::fs::remove_dir_all(&db_path);
}