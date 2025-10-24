use serde::Deserialize;
use mpl_token_metadata::accounts::Metadata;
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_sdk::pubkey::Pubkey;
use std::str::FromStr;
use tracing::{info, warn, error};

#[derive(Debug, Clone, PartialEq)]
pub enum SecurityCheckResult {
    Safe,
    Honeypot,
    NotSellable,
    LowLiquidity,
    HighTaxes,
    CheckFailed(String),
}

#[derive(Deserialize, Debug)]
struct HoneypotIsResponse {
    honeypot: bool,
    #[serde(default)]
    taxes: Taxes,
}

#[derive(Deserialize, Debug, Default)]
struct Taxes {
    buy: f32,
    sell: f32,
}

async fn check_honeypot(client: &reqwest::Client, token_address: &str) -> Result<HoneypotIsResponse, reqwest::Error> {
    let url = format!("https://api.honeypot.is/v2/IsHoneypot?address={}", token_address);
    client.get(&url).send().await?.json::<HoneypotIsResponse>().await
}

async fn check_jupiter_route(client: &reqwest::Client, input_mint: &str, output_mint: &str, amount: u64) -> Result<bool, reqwest::Error> {
    let url = format!(
        "https://quote-api.jup.ag/v6/quote?inputMint={}&outputMint={}&amount={}&slippageBps=500",
        input_mint, output_mint, amount
    );
    let response = client.get(&url).send().await?;
    if !response.status().is_success() {
        return Ok(false);
    }
    let data: serde_json::Value = response.json().await?;
    Ok(data.get("outAmount").and_then(|v| v.as_str()).unwrap_or("0") != "0")
}

pub async fn perform_security_checks(
    bot_core: &crate::core::BotCore,
    token_address: &str,
    trusted_creators: &[Pubkey],
) -> SecurityCheckResult {
    let sol_mint = "So11111111111111111111111111111111111111112";
    let client = bot_core.http_client();

    let (honeypot_res, liquidity_res, sellable_res, _authority_res) = tokio::join!(
        check_honeypot(&client, token_address),
        check_jupiter_route(&client, sol_mint, token_address, 100_000_000),
        check_jupiter_route(&client, token_address, sol_mint, 1_000_000),
        is_token_authority_valid(bot_core, token_address, trusted_creators)
    );

    match honeypot_res {
        Ok(hp) if hp.honeypot => return SecurityCheckResult::Honeypot,
        Ok(hp) if hp.taxes.buy > 0.15 || hp.taxes.sell > 0.15 => return SecurityCheckResult::HighTaxes,
        Err(e) => return SecurityCheckResult::CheckFailed(format!("Honeypot API failed: {}", e)),
        _ => (),
    }

    match liquidity_res {
        Ok(false) => return SecurityCheckResult::LowLiquidity,
        Err(e) => return SecurityCheckResult::CheckFailed(format!("Liquidity check failed: {}", e)),
        _ => (),
    }

    match sellable_res {
        Ok(false) => return SecurityCheckResult::NotSellable,
        Err(e) => return SecurityCheckResult::CheckFailed(format!("Sellability check failed: {}", e)),
        _ => (),
    }

    // if !authority_res {
    //     return SecurityCheckResult::CheckFailed("Untrusted token authority".to_string());
    // }

    info!("[Rust Core] Security checks passed for {}", token_address);
    SecurityCheckResult::Safe
}

async fn is_token_authority_valid(bot_core: &crate::core::BotCore, token_mint_str: &str, trusted_authorities: &[Pubkey]) -> bool {
    if trusted_authorities.is_empty() {
        warn!("[SÉCURITÉ] Aucune autorité de confiance configurée. Tous les tokens seront rejetés.");
        return false;
    }

    let rpc_client = RpcClient::new(bot_core.rpc_url());
    let token_mint_pubkey = match Pubkey::from_str(token_mint_str) {
        Ok(p) => p,
        Err(_) => return false,
    };

    let (metadata_pda, _) = Metadata::find_pda(&token_mint_pubkey);

    match rpc_client.get_account(&metadata_pda).await {
        Ok(account) => {
            if let Ok(metadata) = Metadata::from_bytes(&account.data) {
                if trusted_authorities.contains(&metadata.update_authority) {
                    info!("[SÉCURITÉ] Autorité du token {} validée.", token_mint_str);
                    return true;
                }
            }
        },
        Err(e) => error!("[SÉCURITÉ] Impossible de récupérer le compte de métadonnées pour {}: {}", token_mint_str, e),
    }
    warn!("[SÉCURITÉ] Autorité du token {} INVALIDE ou introuvable.", token_mint_str);
    false
}

#[cfg(test)]
mod tests {
    use super::*;
    #[tokio::test]
    async fn test_honeypot_check_real_call() {
        let client = reqwest::Client::new();
        let result = check_honeypot(&client, "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v").await.map(|r| r.honeypot);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), false);
    }
}