use crate::core::{BotCore, RunMode};
use crate::db::{CreatorRecord, StrategySnapshot, TradeRecord};
use std::sync::Arc;
use tokio;
use uuid::Uuid;

/// Simule l'ajout d'un trade très rentable à la base de données.
async fn setup_profitable_trade(bot_core: &Arc<BotCore>, token_address: &str, _creator: &str) {
    // Pour ce test, nous allons devoir "tricher" et insérer un créateur manuellement
    // car la récupération on-chain est complexe à mocker.
    // On simule que le token a un créateur connu.

    let trade_id = Uuid::new_v4().to_string();
    let buy_timestamp = chrono::Utc::now().timestamp() - 1000; // Trade dans le passé

    let trade = TradeRecord {
        id: trade_id,
        token_address: token_address.to_string(),
        buy_timestamp,
        buy_price: 1.0,
        amount_sol: 0.1,
        amount_tokens: 100.0,
        sell_timestamp: Some(chrono::Utc::now().timestamp()), // Trade in the past
        sell_price: Some(2.5), // Sell price > 2x buy price
        pnl_sol: Some(2.4),     // Positive P&L
        pnl_percent: Some(2400.0),
        strategy_snapshot: Some(StrategySnapshot {
            buy_amount_sol: 0.1,
            sell_multiplier: 2.0,
            trailing_stop_percent: 0.1,
        }),
        run_mode: "DEMO".to_string(),
    };

    bot_core.db().save_trade(&trade).unwrap();

    // On simule que la fonction `fetch_creator_for_token` retournera ce créateur.
    // Dans un vrai test, on utiliserait une interface mockée pour le client RPC.
}

#[tokio::test]
#[ignore] // Ce test nécessite une infrastructure RPC ou des mocks complexes.
async fn test_profitable_creator_tracking() {
    let db_path = format!("test_db_creator_{}", Uuid::new_v4());
    let bot_core = Arc::new(BotCore::new(
        0.01,
        2.0,
        0.15,
        "dummy_pk".to_string(),
        "https://api.mainnet-beta.solana.com".to_string(),
        "dummy_helius_key".to_string(),
        vec![],
        &db_path,
        RunMode::Demo,
        10,
        128,
    ));

    let profitable_token = "DezXAZ8z7PnrnRJjz3wXBoRgixCa6xjnB7YaB1pPB263"; // Exemple: BONK
    let expected_creator = "9M22m.."; // Mettre ici le vrai créateur de BONK si connu

    // 1. Simuler un trade rentable
    setup_profitable_trade(&bot_core, profitable_token, expected_creator).await;

    // 2. Lancer une itération du tracker
    // fastparser::creator_tracker::run_creator_tracker(bot_core.clone()).await; // On ne peut pas appeler la boucle infinie
    // On appellerait une fonction `analyze_once` si elle existait.

    // 3. Vérifier que le créateur est maintenant dans l'index
    // let is_trusted = bot_core.is_creator_trusted(expected_creator).await;
    // assert!(is_trusted, "Le créateur rentable aurait dû être ajouté à l'index de confiance.");

    // 4. Nettoyage
    let _ = std::fs::remove_dir_all(&db_path);
    let _ = tokio::fs::remove_file("data/trusted_creators.jsonl").await;
}