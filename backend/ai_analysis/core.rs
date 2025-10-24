use std::sync::Arc;
use crate::{state_manager::TradingCore, tx_builder, metrics::Metrics};
use solana_sdk::pubkey::Pubkey;
use solana_sdk::signature::Signer;
use std::{collections::HashSet, sync::atomic::{AtomicBool, Ordering}};
use tokio::sync::RwLock;
use tracing::info;
 
#[derive(thiserror::Error, Debug)] // Ensure thiserror is derived
pub enum CoreError {
    #[error("Transaction build failed: {0}")]
    TxBuildError(#[from] tx_builder::TxBuilderError),
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum RunMode {
    Demo,
    Real,
}

#[derive(Clone)]
pub struct BotCore {
    pub trading_core: TradingCore,
    wallet_private_key_bs58: Arc<String>,
    rpc_url: String,
    helius_api_key: String,
    http_client: reqwest::Client,
    pub trusted_creators: Vec<Pubkey>,
    db: crate::db::Database, //NOSONAR
    pub run_mode: Arc<RwLock<RunMode>>,
    pub creator_index: Arc<RwLock<HashSet<String>>>,
    pub trusted_tokens: Arc<RwLock<HashSet<String>>>, //NOSONAR
    is_running: Arc<AtomicBool>,
    #[allow(dead_code)] // This field is intentionally read only once at startup.
    pub initial_run_mode: RunMode,
    metrics: Arc<Metrics>,
    max_positions_real: usize,
    max_positions_demo: usize,
}

impl BotCore {
    pub fn new(
        buy_amount_sol: f64,
        sell_multiplier: f64,
        trailing_stop_percent: f64,
        wallet_private_key_bs58: String,
        rpc_url: String,
        helius_api_key: String,
        trusted_creators: Vec<Pubkey>,
        db_path: &str,
        run_mode: RunMode,
        max_positions_real: usize,
        max_positions_demo: usize,
    ) -> Self {
        Self {
            trading_core: TradingCore::new(buy_amount_sol, sell_multiplier, trailing_stop_percent),
            wallet_private_key_bs58: Arc::new(wallet_private_key_bs58),
            rpc_url,
            helius_api_key,
            http_client: reqwest::Client::new(),
            trusted_creators,
            db: crate::db::Database::new(db_path).expect("Impossible d'ouvrir la base de données"), //NOSONAR
            run_mode: Arc::new(RwLock::new(run_mode)),
            creator_index: Arc::new(RwLock::new(HashSet::new())),
            trusted_tokens: Arc::new(RwLock::new(HashSet::new())),
            is_running: Arc::new(AtomicBool::new(false)), // Le bot ne démarre pas automatiquement
            initial_run_mode: run_mode,
            metrics: Arc::new(Metrics::new()),
            max_positions_real,
            max_positions_demo,
        }
    }

    pub fn trading_core(&self) -> TradingCore {
        self.trading_core.clone()
    }

    pub fn get_wallet_pk(&self) -> String {
        (*self.wallet_private_key_bs58).clone() // This is now public via getter
    }

    pub fn get_wallet_pubkey(&self) -> Result<Pubkey, String> {
        let key_bytes = bs58::decode(&*self.wallet_private_key_bs58)
            .into_vec()
            .map_err(|e| format!("Invalid Base58 in private key: {}", e))?;
        let keypair = solana_sdk::signature::Keypair::try_from(key_bytes.as_slice()).map_err(|e| format!("Invalid key bytes: {}", e))?;
        Ok(keypair.pubkey())
    }

    pub fn rpc_url(&self) -> String {
        self.rpc_url.clone()
    }

    pub fn helius_api_key(&self) -> String {
        self.helius_api_key.clone()
    }

    pub fn http_client(&self) -> reqwest::Client {
        self.http_client.clone()
    }

    pub fn trusted_creators(&self) -> &[Pubkey] {
        &self.trusted_creators
    }

    pub fn db(&self) -> crate::db::Database {
        self.db.clone()
    }

    pub fn get_metrics(&self) -> Arc<Metrics> {
        self.metrics.clone()
    }

    pub async fn is_creator_trusted(&self, creator: &str) -> bool {
        self.creator_index.read().await.contains(creator)
    }

    pub async fn is_token_trusted(&self, token: &str) -> bool {
        self.trusted_tokens.read().await.contains(token)
    }

    pub fn is_running(&self) -> bool {
        self.is_running.load(Ordering::Relaxed)
    }

    pub fn set_running_status(&self, status: bool) {
        self.is_running.store(status, Ordering::Relaxed);
    }

    pub async fn set_run_mode(&self, mode: RunMode) {
        let mut run_mode_guard = self.run_mode.write().await;
        *run_mode_guard = mode;
    }

    pub async fn get_run_mode(&self) -> RunMode {
        *self.run_mode.read().await
    }
    pub async fn get_max_positions(&self) -> usize {
        match *self.run_mode.read().await {
            RunMode::Demo => self.max_positions_demo,
            RunMode::Real => self.max_positions_real,
        }
    }

    pub async fn process_new_token_internal(
        &self,
        token_address: &str,
        quote_response_json: &str,
    ) -> Result<Option<String>, CoreError> {
        if self.trading_core.get_position(token_address).is_some() {
            info!("[Rust Core] Position déjà détenue pour {}, achat ignoré.", token_address);
            return Ok(None);
        }

        info!("[Rust Core] Sécurité OK pour {}. Construction de la transaction...", token_address);
        let build_result = tx_builder::build_and_sign_jupiter_swap_internal(
            &self.http_client,
            quote_response_json.to_string(),
            (*self.wallet_private_key_bs58).clone(),
            self.rpc_url.clone(),
            None,
        )
        .await?;

        Ok(Some(build_result))
    }
}