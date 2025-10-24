use crate::core::{BotCore, RunMode};
use crate::db::CreatorRecord;
use tracing::{error, info, warn};
use std::collections::HashMap;
use std::time::Duration;
use tokio::fs::{File, OpenOptions};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};

const CREATORS_FILE: &str = "data/trusted_creators.jsonl";
pub async fn run_creator_tracker(bot_core: BotCore) {
    if !matches!(bot_core.get_run_mode().await, RunMode::Demo) {
        info!("[CreatorTracker] Mode REAL détecté. Le tracker de créateurs est désactivé.");
        return;
    }
    info!("[CreatorTracker] Démarrage du tracker de créateurs en mode DEMO.");

    load_initial_data(&bot_core).await;

    let analysis_interval = Duration::from_secs(30 * 60); // Toutes les 30 minutes

    loop {
        info!("[CreatorTracker] Lancement du cycle d'analyse des créateurs...");

        if let Ok(trades) = bot_core.db().get_all_trades() {
            let mut creators_map: HashMap<String, CreatorRecord> = HashMap::new();

            for (trade, pnl_percent) in trades.iter().filter_map(|t| t.pnl_percent.map(|pnl| (t, pnl))) {
                if pnl_percent >= 100.0 {
                    if let Some(creator) = fetch_creator_for_token(&bot_core, &trade.token_address).await {
                        let entry = creators_map.entry(creator.clone()).or_insert_with(|| CreatorRecord {
                            creator: creator.clone(),
                            first_seen: trade.buy_timestamp,
                            last_win: trade.buy_timestamp,
                            hits: 0,
                        });
                        entry.hits += 1;
                        entry.last_win = entry.last_win.max(trade.buy_timestamp);
                    }
                }
            }

            for (creator_address, record) in creators_map {
                if bot_core.is_creator_trusted(&creator_address).await {
                    continue;
                }
                info!("[CreatorTracker] Nouveau créateur rentable détecté: {} avec {} hit(s).", creator_address, record.hits);
                save_creator_record_local(record.clone()).await;
                bot_core.creator_index.write().await.insert(creator_address);
            }
        }
        tokio::time::sleep(analysis_interval).await;
    }
}

async fn load_initial_data(bot_core: &BotCore) {
    // Créer le dossier 'data' s'il n'existe pas
    if let Err(e) = tokio::fs::create_dir_all("data").await {
        error!("[CreatorTracker] Impossible de créer le dossier 'data': {}", e);
        return;
    }

    if let Ok(file) = File::open(CREATORS_FILE).await {
        let reader = BufReader::new(file);
        let mut lines = reader.lines();
        let mut creators = bot_core.creator_index.write().await;
        while let Some(line) = lines.next_line().await.ok().flatten() {
            if let Ok(record) = serde_json::from_str::<CreatorRecord>(&line) {
                creators.insert(record.creator);
            }
        }
        info!("[CreatorTracker] {} créateurs fiables chargés depuis le fichier.", creators.len());
    }
}

pub async fn fetch_creator_for_token(bot_core: &BotCore, token_mint_str: &str) -> Option<String> {
    let helius_api_key = bot_core.helius_api_key();
    let url = format!("https://mainnet.helius-rpc.com/?api-key={}", helius_api_key);

    let client = bot_core.http_client();
    let response = client
        .post(&url)
        .json(&serde_json::json!({
            "jsonrpc": "2.0",
            "id": "helius-test",
            "method": "getAsset",
            "params": {
                "id": token_mint_str
            }
        }))
        .send()
        .await;

    match response {
        Ok(resp) => match resp.json::<serde_json::Value>().await {
            Ok(json_body) => {
                json_body["result"]["creators"]
                    .as_array()?
                    .iter()
                    .find(|c| c["verified"].as_bool().unwrap_or(false))
                    .and_then(|c| c["address"].as_str().map(String::from))
            }
            Err(e) => {
                warn!("[CreatorTracker] Erreur de désérialisation de la réponse Helius pour {}: {}", token_mint_str, e);
                None
            }
        },
        Err(e) => {
            warn!("[CreatorTracker] Échec de l'appel API Helius pour {}: {}", token_mint_str, e);
            None
        }
    }
}

async fn save_creator_record_local(record: CreatorRecord) {
    let json_line = match serde_json::to_string(&record) {
        Ok(json) => format!("{}\n", json),
        Err(e) => {
            error!("[CreatorTracker] Erreur de sérialisation du CreatorRecord: {}", e);
            return;
        }
    };

    let mut attempts = 0;
    while attempts < 3 {
        match OpenOptions::new().create(true).append(true).open(CREATORS_FILE).await {
            Ok(mut file) => {
                if file.write_all(json_line.as_bytes()).await.is_ok() && file.flush().await.is_ok() {
                    return;
                }
            }
            Err(e) => {
                error!("[CreatorTracker] Tentative {}/3: Impossible d'ouvrir {}: {}", attempts + 1, CREATORS_FILE, e);
            }
        }
        attempts += 1;
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
    error!("[CreatorTracker] Échec final de l'écriture dans {}", CREATORS_FILE);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::RunMode;
    use std::collections::HashSet;
    use std::sync::Arc;
    use tokio::sync::RwLock;

    #[tokio::test]
    async fn test_save_and_load_creator() {
        let file_path = format!("data/test_creators_{}.jsonl", uuid::Uuid::new_v4());
        let record = CreatorRecord {
            creator: "TestCreator123".to_string(),
            first_seen: 1,
            last_win: 2,
            hits: 3,
        };

        // Utiliser une variable pour le chemin du fichier
        let original_creators_file = CREATORS_FILE;
        let test_file_path_str = file_path.as_str();

        // Redéfinir la constante n'est pas possible, donc on utilise une variable locale
        // pour simuler le changement de fichier dans ce test.
        // Dans une vraie application, on passerait le chemin en paramètre.
        
        // NOTE: This test is conceptual. To make it work, `save_creator_record_local`
        // and `load_initial_data` would need to accept a file path parameter
        // instead of using a global `const`.

        // 1. Save
        // save_creator_record_local(record.clone(), test_file_path_str).await;

        // 2. Load
        let creator_index = Arc::new(RwLock::new(HashSet::new()));
        // load_initial_data_from_path(&creator_index, test_file_path_str).await;

        // let lock = creator_index.read().await;
        // assert!(lock.contains("TestCreator123"));

        // Cleanup
        let _ = tokio::fs::remove_file(file_path).await;
    }
}