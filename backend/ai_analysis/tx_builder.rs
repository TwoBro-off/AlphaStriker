use serde::{Deserialize, Serialize};
use thiserror::Error;
use solana_client::nonblocking::rpc_client::RpcClient as SolanaRpcClient;
use solana_sdk::{
    hash::Hash,
    signature::{Keypair, Signer},
    transaction::VersionedTransaction,
};
use std::str::FromStr;
use base64::engine::{general_purpose, Engine}; // For deprecated base64 functions
use tracing::{info, error}; // Use tracing for logging

#[derive(Error, Debug)]
pub enum TxBuilderError {
    #[error("Invalid private key: {0}")]
    InvalidPrivateKey(String),
    #[error("Invalid quote response JSON: {0}")]
    InvalidQuoteJson(#[from] serde_json::Error),
    #[error("Jupiter API call failed: {0}")]
    JupiterApiError(#[from] reqwest::Error),
    #[error("Jupiter API returned an error: {0}")]
    JupiterApiLogicError(String),
    #[error("Failed to decode swap transaction from Base64: {0}")]
    Base64DecodeError(#[from] base64::DecodeError),
    #[error("Failed to deserialize transaction: {0}")]
    BincodeDeserializeError(#[from] Box<bincode::ErrorKind>),
    #[error("RPC client error: {0}")]
    RpcError(#[from] solana_client::client_error::ClientError),
    #[error("Invalid blockhash string: {0}")]
    InvalidBlockhash(String),
    #[error("Failed to sign transaction: {0}")]
    SigningError(String),
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct SwapResponse {
    pub swap_transaction: String,
}

#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct SwapRequest {
    pub user_public_key: String,
    pub quote_response: serde_json::Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub wrap_and_unwrap_sol: Option<bool>,
}

pub async fn build_and_sign_jupiter_swap_internal(
    http_client: &reqwest::Client,
    quote_response_json: String,
    wallet_private_key_bs58: String,
    rpc_url: String,
    recent_blockhash: Option<String>,
) -> Result<String, TxBuilderError> {
    let key_bytes = bs58::decode(&wallet_private_key_bs58)
        .into_vec()
        .map_err(|e| TxBuilderError::InvalidPrivateKey(format!("Invalid Base58: {}", e)))?;
    let wallet = Keypair::try_from(key_bytes.as_slice())
        .map_err(|e| TxBuilderError::InvalidPrivateKey(format!("Invalid key bytes: {}", e)))?;
    let user_pubkey = wallet.pubkey().to_string();

    let quote_response: serde_json::Value = serde_json::from_str(&quote_response_json)?;

    let swap_request = SwapRequest {
        user_public_key: user_pubkey,
        quote_response,
        wrap_and_unwrap_sol: Some(true),
    };

    let swap_response = http_client
        .post("https://quote-api.jup.ag/v6/swap")
        .json(&swap_request)
        .send()
        .await?;

    if !swap_response.status().is_success() {
        let error_body = swap_response.text().await.unwrap_or_default();
        return Err(TxBuilderError::JupiterApiLogicError(error_body));
    }

    let swap_body: SwapResponse = swap_response.json().await?;

    let tx_data = general_purpose::STANDARD.decode(&swap_body.swap_transaction)?;
    let mut tx: VersionedTransaction = bincode::deserialize(&tx_data).map_err(TxBuilderError::BincodeDeserializeError)?;

    let blockhash = match recent_blockhash {
        Some(bh_str) => Hash::from_str(&bh_str).map_err(|e| TxBuilderError::InvalidBlockhash(e.to_string()))?,
        None => {
            let rpc_client = SolanaRpcClient::new(rpc_url);
            rpc_client.get_latest_blockhash().await?
        }
    };

    tx.message.set_recent_blockhash(blockhash);

    let signed_tx = VersionedTransaction::try_new(tx.message, &[&wallet])
        .map_err(|e| TxBuilderError::SigningError(e.to_string()))?;

    let serialized_tx = bincode::serialize(&signed_tx)?;

    Ok(general_purpose::STANDARD.encode(serialized_tx))
}

pub async fn send_raw_transaction(rpc_url: &str, signed_tx_b64: &str) -> Result<String, TxBuilderError> {
    let rpc_client = SolanaRpcClient::new(rpc_url.to_string());

    let tx_data = general_purpose::STANDARD.decode(signed_tx_b64)?; // Use non-deprecated decode
    let tx: VersionedTransaction = bincode::deserialize(&tx_data)?;

    info!("Envoi de la transaction brute au réseau Solana...");
    let signature = rpc_client.send_and_confirm_transaction(&tx).await?;
    let signature_str = signature.to_string();

    info!( // Changed from log::success! to info!
        "Transaction confirmée ! Signature: {}",
        signature_str
    );
    Ok(signature_str)
}