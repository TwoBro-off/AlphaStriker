use crate::core::BotCore;
use crate::log_scanner::handle_new_token;
use std::time::Duration;
use tokio::time::sleep;
use tracing::info;

// Tokens populaires pour tester le bot
const TEST_TOKENS: &[&str] = &[
    "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v", // USDC
    "Es9vMFrzaCERmJfrF4H2FYD4KCoNkY11McCe8BenwNYB",  // USDT
    "DezXAZ8z7PnrnRJjz3wXBoRgixCa6xjnB7YaB1pPB263",  // Bonk
    "So11111111111111111111111111111111111111112",     // SOL
];

pub async fn run_test_mode(bot_core: BotCore) {
    info!("[Test Mode] Démarrage du mode de test avec des tokens existants");
    
    for (i, token) in TEST_TOKENS.iter().enumerate() {
        info!("[Test Mode] Test du token {}: {}", i + 1, token);
        
        let core_clone = bot_core.clone();
        let token_clone = token.to_string();
        
        tokio::spawn(async move {
            handle_new_token(core_clone, token_clone).await;
        });
        
        // Attendre entre chaque test
        sleep(Duration::from_secs(10)).await;
    }
    
    info!("[Test Mode] Tests terminés");
}
