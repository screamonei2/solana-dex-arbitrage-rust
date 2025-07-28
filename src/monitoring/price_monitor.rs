use anyhow::Result;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, error, debug};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use solana_sdk::pubkey::Pubkey;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::config::BotConfig;
use crate::dex::{DexClient, Quote, DexType};
use crate::utils::current_timestamp_ms;

// ============================================================================
// PRICE DATA STRUCTURES
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PriceData {
    pub token_mint: Pubkey,
    pub price_usd: f64,
    pub price_sol: f64,
    pub volume_24h: f64,
    pub market_cap: f64,
    pub timestamp: u64,
    pub source: String,
}

impl PriceData {
    pub fn new(
        token_mint: Pubkey,
        price_usd: f64,
        price_sol: f64,
        source: String,
    ) -> Self {
        Self {
            token_mint,
            price_usd,
            price_sol,
            volume_24h: 0.0,
            market_cap: 0.0,
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            source,
        }
    }

    pub fn is_stale(&self, max_age_seconds: u64) -> bool {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        
        now - self.timestamp > max_age_seconds
    }
}

#[derive(Debug, Clone)]
pub struct PriceUpdate {
    pub token_mint: Pubkey,
    pub old_price: f64,
    pub new_price: f64,
    pub change_percent: f64,
    pub timestamp: u64,
}

// ============================================================================
// PRICE MONITOR
// ============================================================================

pub struct PriceMonitor {
    prices: Arc<RwLock<HashMap<Pubkey, PriceData>>>,
    update_interval_ms: u64,
    max_price_age_seconds: u64,
}

impl PriceMonitor {
    pub fn new(
        update_interval_ms: u64,
        max_price_age_seconds: u64,
    ) -> Self {
        Self {
            prices: Arc::new(RwLock::new(HashMap::new())),
            update_interval_ms,
            max_price_age_seconds,
        }
    }

    pub async fn start(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // This would connect to price feeds and start monitoring
        // For now, just return Ok
        Ok(())
    }

    pub async fn get_price(&self, token_mint: &Pubkey) -> Option<PriceData> {
        let prices = self.prices.read().await;
        prices.get(token_mint).cloned()
    }

    pub async fn update_price(&self, price_data: PriceData) {
        let mut prices = self.prices.write().await;
        prices.insert(price_data.token_mint, price_data);
    }

    pub async fn get_all_prices(&self) -> HashMap<Pubkey, PriceData> {
        let prices = self.prices.read().await;
        prices.clone()
    }

    pub async fn cleanup_stale_prices(&self) {
        let mut prices = self.prices.write().await;
        prices.retain(|_, price_data| !price_data.is_stale(self.max_price_age_seconds));
    }

    pub fn get_update_interval(&self) -> u64 {
        self.update_interval_ms
    }
} 